use bytemuck::{Pod, Zeroable};
use std::{
    borrow::Cow, cell::RefCell, collections::BTreeMap, f32::consts, marker::PhantomData, mem,
    num::NonZeroU8, rc::Rc, sync::Arc,
};
use wgpu::{util::DeviceExt, Instance, Surface};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, StartCause, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::{Key, NamedKey},
    window::Window,
};

#[cfg(target_os = "windows")]
use winapi::um::winuser::SetCursorPos;

#[cfg(target_os = "macos")]
use core_graphics::{
    display::CGDisplay,
    event::{CGEvent, CGEventType, CGMouseButton},
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::CGPoint,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod input;
mod map;

use input::Input;
use map::{Chunk, Map, CHUNK_SIZE};

const REGIONS: [[i32; 3]; 8] = [
    [-1, -1, -1],
    [-1, -1, 0],
    [-1, 0, -1],
    [-1, 0, 0],
    [0, -1, -1],
    [0, -1, 0],
    [0, 0, -1],
    [0, 0, 0],
];

struct EventLoopWrapper {
    event_loop: EventLoop<()>,
    window: Arc<Window>,
    #[cfg(target_arch = "wasm32")]
    canvas: Option<web_sys::Element>,
    #[cfg(not(target_arch = "wasm32"))]
    canvas: Option<PhantomData<NonZeroU8>>,
}

impl EventLoopWrapper {
    pub fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        let mut builder = winit::window::WindowBuilder::new();
        builder = builder.with_title("ft_vox");
        let window = Arc::new(builder.build(&event_loop).unwrap());

        let mut outer_canvas = None;
        #[cfg(target_arch = "wasm32")]
        {
            use winit::dpi::PhysicalSize;
            let _ = window.request_inner_size(PhysicalSize::new(450, 400));

            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-container")?;
                    let canvas = web_sys::Element::from(window.canvas()?);
                    dst.append_child(&canvas).ok()?;
                    {
                        let canvas_clone = canvas.clone();
                        let closure = Closure::wrap(Box::new(move || {
                            canvas_clone.request_pointer_lock();
                        }) as Box<dyn FnMut()>);
                        canvas
                            .add_event_listener_with_callback(
                                "click",
                                closure.as_ref().unchecked_ref(),
                            )
                            .expect("Failed to add click event listener");
                        closure.forget();
                    }
                    outer_canvas = Some(canvas);
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        Self {
            event_loop,
            window,
            canvas: outer_canvas,
        }
    }
}

/// Wrapper type which manages the surface and surface configuration.
///
/// As surface usage varies per platform, wrapping this up cleans up the event loop code.
struct SurfaceWrapper {
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
}

impl SurfaceWrapper {
    /// Create a new surface wrapper with no surface or configuration.
    fn new() -> Self {
        Self {
            surface: None,
            config: None,
        }
    }

    /// Called after the instance is created, but before we request an adapter.
    ///
    /// On wasm, we need to create the surface here, as the WebGL backend needs
    /// a surface (and hence a canvas) to be present to create the adapter.
    ///
    /// We cannot unconditionally create a surface here, as Android requires
    /// us to wait until we receive the `Resumed` event to do so.
    fn pre_adapter(&mut self, instance: &Instance, window: Arc<Window>) {
        if cfg!(target_arch = "wasm32") {
            self.surface = Some(instance.create_surface(window).unwrap());
        }
    }

    /// Check if the event is the start condition for the surface.
    fn start_condition(e: &Event<()>) -> bool {
        match e {
            // On all other platforms, we can create the surface immediately.
            Event::NewEvents(StartCause::Init) => !cfg!(target_os = "android"),
            // On android we need to wait for a resumed event to create the surface.
            Event::Resumed => cfg!(target_os = "android"),
            _ => false,
        }
    }

    /// Called when an event which matches [`Self::start_condition`] is received.
    ///
    /// On all native platforms, this is where we create the surface.
    ///
    /// Additionally, we configure the surface based on the (now valid) window size.
    fn resume(&mut self, context: &Context, window: Arc<Window>, srgb: bool) {
        // Window size is only actually valid after we enter the event loop.
        let window_size = window.inner_size();
        let width = window_size.width.max(1);
        let height = window_size.height.max(1);

        log::info!("Surface resume {window_size:?}");

        // We didn't create the surface in pre_adapter, so we need to do so now.
        if !cfg!(target_arch = "wasm32") {
            self.surface = Some(context.instance.create_surface(window).unwrap());
        }

        // From here on, self.surface should be Some.

        let surface = self.surface.as_ref().unwrap();

        // Get the default configuration,
        let mut config = surface
            .get_default_config(&context.adapter, width, height)
            .expect("Surface isn't supported by the adapter.");
        if srgb {
            // Not all platforms (WebGPU) support sRGB swapchains, so we need to use view formats
            let view_format = config.format.add_srgb_suffix();
            config.view_formats.push(view_format);
        } else {
            // All platforms support non-sRGB swapchains, so we can just use the format directly.
            let format = config.format.remove_srgb_suffix();
            config.format = format;
            config.view_formats.push(format);
        };

        surface.configure(&context.device, &config);
        self.config = Some(config);
    }

    /// Resize the surface, making sure to not resize to zero.
    fn resize(&mut self, context: &Context, size: PhysicalSize<u32>) {
        log::info!("Surface resize {size:?}");

        let config = self.config.as_mut().unwrap();
        config.width = size.width;
        config.height = size.height;
        #[cfg(target_arch = "wasm32")]
        {
            let device_pixel_ratio = web_sys::window().unwrap().device_pixel_ratio();
            config.width = (config.width as f64 / device_pixel_ratio) as u32;
            config.height = (config.height as f64 / device_pixel_ratio) as u32;
        }
        config.width = config.width.max(1);
        config.height = config.height.max(1);
        let surface = self.surface.as_ref().unwrap();
        surface.configure(&context.device, config);
    }

    /// Acquire the next surface texture.
    fn acquire(&mut self, context: &Context) -> wgpu::SurfaceTexture {
        let surface = self.surface.as_ref().unwrap();

        match surface.get_current_texture() {
            Ok(frame) => frame,
            // If we timed out, just try again
            Err(wgpu::SurfaceError::Timeout) => surface
                .get_current_texture()
                .expect("Failed to acquire next surface texture!"),
            Err(
                // If the surface is outdated, or was lost, reconfigure it.
                wgpu::SurfaceError::Outdated
                | wgpu::SurfaceError::Lost
                // If OutOfMemory happens, reconfiguring may not help, but we might as well try
                | wgpu::SurfaceError::OutOfMemory,
            ) => {
                surface.configure(&context.device, self.config());
                surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture!")
            }
        }
    }

    /// On suspend on android, we drop the surface, as it's no longer valid.
    ///
    /// A suspend event is always followed by at least one resume event.
    fn suspend(&mut self) {
        if cfg!(target_os = "android") {
            self.surface = None;
        }
    }

    fn get(&self) -> Option<&Surface> {
        self.surface.as_ref()
    }

    fn config(&self) -> &wgpu::SurfaceConfiguration {
        self.config.as_ref().unwrap()
    }
}

/// Context containing global wgpu resources.
struct Context {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}
impl Context {
    /// Initializes the example context.
    async fn init_async(surface: &mut SurfaceWrapper, window: Arc<Window>) -> Self {
        let backends = wgpu::util::backend_bits_from_env().unwrap_or_default();
        let dx12_shader_compiler = wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();
        let gles_minor_version = wgpu::util::gles_minor_version_from_env().unwrap_or_default();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            flags: wgpu::InstanceFlags::from_build_config().with_env(),
            dx12_shader_compiler,
            gles_minor_version,
        });
        surface.pre_adapter(&instance, window);
        let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, surface.get())
            .await
            .expect("No suitable GPU adapters found on the system!");

        // let adapter_info = adapter.get_info();
        // log::info!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

        let optional_features = wgpu::Features::empty();
        let required_features = wgpu::Features::empty();
        let adapter_features = adapter.features();
        assert!(
            adapter_features.contains(required_features),
            "Adapter does not support required features for this example: {:?}",
            required_features - adapter_features
        );

        let required_downlevel_capabilities = wgpu::DownlevelCapabilities {
            flags: wgpu::DownlevelFlags::empty(),
            shader_model: wgpu::ShaderModel::Sm5,
            ..wgpu::DownlevelCapabilities::default()
        };
        let downlevel_capabilities = adapter.get_downlevel_capabilities();
        assert!(
            downlevel_capabilities.shader_model >= required_downlevel_capabilities.shader_model,
            "Adapter does not support the minimum shader model required to run this example: {:?}",
            required_downlevel_capabilities.shader_model
        );
        assert!(
            downlevel_capabilities
                .flags
                .contains(required_downlevel_capabilities.flags),
            "Adapter does not support the downlevel capabilities required to run this example: {:?}",
            required_downlevel_capabilities.flags - downlevel_capabilities.flags
        );

        // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the surface.
        let needed_limits =
            wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());

        let trace_dir = std::env::var("WGPU_TRACE");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: (optional_features & adapter_features) | required_features,
                    required_limits: needed_limits,
                    ..Default::default()
                },
                trace_dir.ok().as_ref().map(std::path::Path::new),
            )
            .await
            .expect("Unable to find a suitable GPU adapter!");

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Debug).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let window_loop = EventLoopWrapper::new();
    let mut surface = SurfaceWrapper::new();
    let context = Context::init_async(&mut surface, window_loop.window.clone()).await;
    let vox: Rc<RefCell<Option<Vox>>> = Rc::new(RefCell::new(None));
    #[cfg(target_arch = "wasm32")]
    {
        let vox = Rc::clone(&vox);

        let sensitive: f32 = 0.0015;
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if let Some(vox) = vox.borrow_mut().as_mut() {
                let delta_x = event.movement_x() as f64;
                let delta_y = event.movement_y() as f64;

                vox.horizontal_rotation -= delta_x as f32 * sensitive;
                vox.horizontal_rotation %= 2.0 * std::f32::consts::PI;
                if vox.horizontal_rotation < 0.0 {
                    vox.horizontal_rotation += 2.0 * std::f32::consts::PI;
                }

                vox.vertical_rotation -= delta_y as f32 * sensitive;
                vox.vertical_rotation = vox
                    .vertical_rotation
                    .clamp(-0.5 * std::f32::consts::PI, 0.5 * std::f32::consts::PI);
            }
        }) as Box<dyn FnMut(_)>);
        window_loop
            .canvas
            .unwrap()
            .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())
            .expect("Failed to add mousemove event listener");
        closure.forget();
    }
    let mut input = Input::new();
    let event_loop_function = EventLoop::run;

    #[allow(clippy::let_unit_value)]
    let _ = (event_loop_function)(
        window_loop.event_loop,
        move |event: Event<()>, target: &EventLoopWindowTarget<()>| {
            match event {
                ref e if SurfaceWrapper::start_condition(e) => {
                    surface.resume(&context, window_loop.window.clone(), false);

                    // If we haven't created the example yet, do so now.
                    if vox.borrow().is_none() {
                        *vox.borrow_mut() = Some(Vox::init(
                            surface.config(),
                            &context.adapter,
                            &context.device,
                            &context.queue,
                        ));
                    }
                }
                Event::Suspended => {
                    surface.suspend();
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        surface.resize(&context, size);
                        if let Some(vox) = vox.borrow_mut().as_mut() {
                            vox.resize(surface.config(), &context.device, &context.queue);
                        }
                        window_loop.window.request_redraw();
                    }
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    }
                    | WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: Key::Character(s),
                                ..
                            },
                        ..
                    } if s == "r" => {
                        println!("{:#?}", context.instance.generate_report());
                    }

                    // key pressed
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key,
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => match logical_key {
                        Key::Character(s) => match s.as_str() {
                            "w" => {
                                input.key_w = true;
                            }
                            "a" => {
                                input.key_a = true;
                            }
                            "s" => {
                                input.key_s = true;
                            }
                            "d" => {
                                input.key_d = true;
                            }
                            _ => {}
                        },
                        Key::Named(NamedKey::Shift) => {
                            input.key_shift = true;
                        }
                        Key::Named(NamedKey::Space) => {
                            input.key_space = true;
                        }
                        _ => {}
                    },

                    // WASD key released
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key,
                                state: ElementState::Released,
                                ..
                            },
                        ..
                    } => match logical_key {
                        Key::Character(s) => match s.as_str() {
                            "w" => {
                                input.key_w = false;
                            }
                            "a" => {
                                input.key_a = false;
                            }
                            "s" => {
                                input.key_s = false;
                            }
                            "d" => {
                                input.key_d = false;
                            }
                            _ => {}
                        },
                        Key::Named(NamedKey::Shift) => {
                            input.key_shift = false;
                        }
                        Key::Named(NamedKey::Space) => {
                            input.key_space = false;
                        }
                        _ => {}
                    },

                    WindowEvent::CursorMoved {
                        position: local_cursor_position,
                        ..
                    } => input.local_cursor_position = local_cursor_position,

                    WindowEvent::RedrawRequested => {
                        // On MacOS, currently redraw requested comes in _before_ Init does.
                        // If this happens, just drop the requested redraw on the floor.
                        //
                        // See https://github.com/rust-windowing/winit/issues/3235 for some discussion

                        if let Some(vox) = vox.borrow_mut().as_mut() {
                            // Movement by keyboard
                            {
                                if input.key_w && !input.key_s {
                                    let forward_x = -vox.horizontal_rotation.sin();
                                    let forward_y = vox.horizontal_rotation.cos();
                                    vox.eye.x += forward_x * 0.1;
                                    vox.eye.y += forward_y * 0.1;
                                }

                                if input.key_a && !input.key_d {
                                    let forward_x = -vox.horizontal_rotation.sin();
                                    let forward_y = vox.horizontal_rotation.cos();
                                    let leftward_x = -forward_y;
                                    let leftward_y = forward_x;
                                    vox.eye.x += leftward_x * 0.1;
                                    vox.eye.y += leftward_y * 0.1;
                                }

                                if input.key_s && !input.key_w {
                                    let forward_x = -vox.horizontal_rotation.sin();
                                    let forward_y = vox.horizontal_rotation.cos();
                                    vox.eye.x -= forward_x * 0.1;
                                    vox.eye.y -= forward_y * 0.1;
                                }

                                if input.key_d && !input.key_a {
                                    let forward_x = -vox.horizontal_rotation.sin();
                                    let forward_y = vox.horizontal_rotation.cos();
                                    let rightward_x = forward_y;
                                    let rightward_y = -forward_x;
                                    vox.eye.x += rightward_x * 0.1;
                                    vox.eye.y += rightward_y * 0.1;
                                }

                                if input.key_space && !input.key_shift {
                                    vox.eye.z += 0.1;
                                }

                                if input.key_shift && !input.key_space {
                                    vox.eye.z -= 0.1;
                                }
                            }

                            // Rotation by mouse
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                let sensitive: f32 = 0.0015;
                                if let Ok(window_position) = window_loop.window.inner_position() {
                                    let window_size = window_loop.window.inner_size();
                                    let delta_x = input.local_cursor_position.x
                                        - (window_size.width / 2) as f64;
                                    let delta_y = input.local_cursor_position.y
                                        - (window_size.height / 2) as f64;
                                    vox.horizontal_rotation -= delta_x as f32 * sensitive;
                                    vox.horizontal_rotation %= 2.0 * std::f32::consts::PI;
                                    if vox.horizontal_rotation < 0.0 {
                                        vox.horizontal_rotation += 2.0 * std::f32::consts::PI;
                                    }

                                    vox.vertical_rotation -= delta_y as f32 * sensitive;
                                    vox.vertical_rotation = vox.vertical_rotation.clamp(
                                        -0.5 * std::f32::consts::PI,
                                        0.5 * std::f32::consts::PI,
                                    );

                                    let center_x: i32 =
                                        window_position.x + (window_size.width / 2) as i32;
                                    let center_y: i32 =
                                        window_position.y + (window_size.height / 2) as i32;

                                    #[cfg(target_os = "windows")]
                                    unsafe {
                                        SetCursorPos(center_x, center_y);
                                    }

                                    #[cfg(target_os = "macos")]
                                    {
                                        let display_size_os =
                                            target.primary_monitor().unwrap().size();
                                        let display_size_cg = CGDisplay::main().bounds().size;
                                        let scaling_factor =
                                            display_size_cg.width / display_size_os.width as f64;
                                        let scaled_x = center_x as f64 * scaling_factor;
                                        let scaled_y = center_y as f64 * scaling_factor;
                                        let source = CGEventSource::new(
                                            CGEventSourceStateID::HIDSystemState,
                                        )
                                        .unwrap();
                                        let event = CGEvent::new_mouse_event(
                                            source,
                                            CGEventType::MouseMoved,
                                            CGPoint::new(scaled_x, scaled_y),
                                            CGMouseButton::Left,
                                        )
                                        .unwrap();
                                        event.post(core_graphics::event::CGEventTapLocation::HID);
                                    }
                                }
                            }
                            let frame = surface.acquire(&context);
                            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
                                format: Some(surface.config().view_formats[0]),
                                ..wgpu::TextureViewDescriptor::default()
                            });

                            vox.render(&view, &context.device, &context.queue);

                            frame.present();

                            window_loop.window.request_redraw();
                        }
                    }
                    _ => {
                        if let Some(vox) = vox.borrow_mut().as_mut() {
                            vox.update(event);
                        }
                    }
                },
                _ => {}
            }
        },
    );
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}

fn vertex(pos: [f32; 3], tc: [f32; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0], pos[1], pos[2], 1.0],
        _tex_coord: [tc[0], tc[1]],
    }
}

fn create_vertices_for_chunk(
    chunk: &Chunk,
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    chunk_px: &Chunk,
    chunk_nx: &Chunk,
    chunk_py: &Chunk,
    chunk_ny: &Chunk,
    chunk_pz: &Chunk,
    chunk_nz: &Chunk,
) -> (Vec<Vertex>, Vec<u16>) {
    let x_offset = chunk_x * CHUNK_SIZE as i32;
    let y_offset = chunk_y * CHUNK_SIZE as i32;
    let z_offset = chunk_z * CHUNK_SIZE as i32;

    let mut vertex_data = Vec::<Vertex>::new();
    let mut index_data = Vec::<u16>::new();
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if chunk.cubes[z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x].is_solid() {
                    let actual_x = x_offset + x as i32;
                    let actual_y = y_offset + y as i32;
                    let actual_z = z_offset + z as i32;
                    let (mut tmp_vertex_data, mut tmp_index_data) = create_vertices(
                        actual_x as f32,
                        actual_y as f32,
                        actual_z as f32,
                        if x == CHUNK_SIZE - 1 {
                            chunk_px.cubes[z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE].is_solid()
                        } else {
                            chunk.cubes[z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x + 1]
                                .is_solid()
                        },
                        if x == 0 {
                            chunk_nx.cubes
                                [z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + CHUNK_SIZE - 1]
                                .is_solid()
                        } else {
                            chunk.cubes[z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x - 1]
                                .is_solid()
                        },
                        if y == CHUNK_SIZE - 1 {
                            chunk_py.cubes[z * CHUNK_SIZE * CHUNK_SIZE + x].is_solid()
                        } else {
                            chunk.cubes[z * CHUNK_SIZE * CHUNK_SIZE + (y + 1) * CHUNK_SIZE + x]
                                .is_solid()
                        },
                        if y == 0 {
                            chunk_ny.cubes
                                [z * CHUNK_SIZE * CHUNK_SIZE + (CHUNK_SIZE - 1) * CHUNK_SIZE + x]
                                .is_solid()
                        } else {
                            chunk.cubes[z * CHUNK_SIZE * CHUNK_SIZE + (y - 1) * CHUNK_SIZE + x]
                                .is_solid()
                        },
                        if z == CHUNK_SIZE - 1 {
                            chunk_pz.cubes[y * CHUNK_SIZE + x].is_solid()
                        } else {
                            chunk.cubes[(z + 1) * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x]
                                .is_solid()
                        },
                        if z == 0 {
                            chunk_nz.cubes
                                [(CHUNK_SIZE - 1) * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x]
                                .is_solid()
                        } else {
                            chunk.cubes[(z - 1) * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x]
                                .is_solid()
                        },
                        vertex_data.len(),
                    );
                    vertex_data.append(&mut tmp_vertex_data);
                    index_data.append(&mut tmp_index_data);
                }
            }
        }
    }
    (vertex_data, index_data)
}

fn create_vertices(
    x: f32,
    y: f32,
    z: f32,
    px: bool,
    nx: bool,
    py: bool,
    ny: bool,
    pz: bool,
    nz: bool,
    index: usize,
) -> (Vec<Vertex>, Vec<u16>) {
    let offset = index as u16;

    let mut vertex_data = Vec::<Vertex>::new();
    let mut index_data = Vec::<u16>::new();

    if !px {
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 0.0], [0.0, 0.0]));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 0.0], [1.0, 0.0]));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 1.0], [1.0, 1.0]));
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 1.0], [0.0, 1.0]));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }

    if !nx {
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 1.0], [1.0, 0.0]));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 1.0], [0.0, 0.0]));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 0.0], [0.0, 1.0]));
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 0.0], [1.0, 1.0]));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }

    if !py {
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 0.0], [1.0, 0.0]));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 0.0], [0.0, 0.0]));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 1.0], [0.0, 1.0]));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 1.0], [1.0, 1.0]));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }

    if !ny {
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 1.0], [0.0, 0.0]));
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 1.0], [1.0, 0.0]));
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 0.0], [1.0, 1.0]));
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 0.0], [0.0, 1.0]));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }

    if !pz {
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 1.0], [0.0, 0.0]));
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 1.0], [1.0, 0.0]));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 1.0], [1.0, 1.0]));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 1.0], [0.0, 1.0]));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }

    if !nz {
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 0.0], [1.0, 0.0]));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 0.0], [0.0, 0.0]));
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 0.0], [0.0, 1.0]));
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 0.0], [1.0, 1.0]));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }

    (vertex_data, index_data)
}

fn create_texels(size: usize) -> Vec<u8> {
    (0..size * size)
        .map(|id| {
            // get high five for recognizing this ;)
            let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
            let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
            let (mut x, mut y, mut count) = (cx, cy, 0);
            while count < 0xFF && x * x + y * y < 4.0 {
                let old_x = x;
                x = x * x - y * y + cx;
                y = 2.0 * old_x * y + cy;
                count += 1;
            }
            count
        })
        .collect()
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    transform: [[f32; 4]; 4], // This represents a single transformation matrix.
}

struct Vox {
    eye: glam::Vec3,
    horizontal_rotation: f32,
    vertical_rotation: f32,
    projection_matrix: glam::Mat4,
    depth_buffer: wgpu::TextureView,
    map: Map,
    chunks: BTreeMap<[i32; 3], Rc<Chunk>>,
    buffers: BTreeMap<[i32; 3], Rc<(wgpu::Buffer, wgpu::Buffer, u32)>>,
    bind_group: wgpu::BindGroup,
    uniform_vp_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl Vox {
    fn get_chunk(&mut self, x: i32, y: i32, z: i32) -> Rc<Chunk> {
        if !self.chunks.contains_key(&[x, y, z]) {
            let result = Rc::new(self.map.get_chunk(x, y, z));
            self.chunks.insert([x, y, z], result);
        }
        Rc::clone(self.chunks.get(&[x, y, z]).unwrap())
    }

    fn get_buffers(
        &mut self,
        device: &wgpu::Device,
        x: i32,
        y: i32,
        z: i32,
    ) -> Rc<(wgpu::Buffer, wgpu::Buffer, u32)> {
        if !self.buffers.contains_key(&[x, y, z]) {
            let result = Rc::new(self.create_buffers(device, x, y, z));
            self.buffers.insert([x, y, z], result);
        }
        Rc::clone(self.buffers.get(&[x, y, z]).unwrap())
    }

    fn create_buffers(
        &mut self,
        device: &wgpu::Device,
        x: i32,
        y: i32,
        z: i32,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let chunk = self.get_chunk(x, y, z);
        let chunk_px = self.get_chunk(x + 1, y, z);
        let chunk_nx = self.get_chunk(x - 1, y, z);
        let chunk_py = self.get_chunk(x, y + 1, z);
        let chunk_ny = self.get_chunk(x, y - 1, z);
        let chunk_pz = self.get_chunk(x, y, z + 1);
        let chunk_nz = self.get_chunk(x, y, z - 1);

        let (vertex_data, index_data) = create_vertices_for_chunk(
            &chunk, x, y, z, &chunk_px, &chunk_nx, &chunk_py, &chunk_ny, &chunk_pz, &chunk_nz,
        );

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        return (vertex_buf, index_buf, index_data.len() as u32);
    }

    fn generate_projection_matrix(aspect_ratio: f32) -> glam::Mat4 {
        glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 1000.0)
    }

    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let texture_extent = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let draw_depth_buffer = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Buffer"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_buffer = draw_depth_buffer.create_view(&wgpu::TextureViewDescriptor::default());

        let map = Map::new(42);
        let chunk_x = 0;
        let chunk_y = 0;
        let chunk_z = 0;
        let chunk = map.get_chunk(chunk_x, chunk_y, chunk_z);
        let chunk_px = map.get_chunk(chunk_x + 1, chunk_y, chunk_z);
        let chunk_nx = map.get_chunk(chunk_x - 1, chunk_y, chunk_z);
        let chunk_py = map.get_chunk(chunk_x, chunk_y + 1, chunk_z);
        let chunk_ny = map.get_chunk(chunk_x, chunk_y - 1, chunk_z);
        let chunk_pz = map.get_chunk(chunk_x, chunk_y, chunk_z + 1);
        let chunk_nz = map.get_chunk(chunk_x, chunk_y, chunk_z - 1);
        let (vertex_data, index_data) = create_vertices_for_chunk(
            &chunk, chunk_x, chunk_y, chunk_z, &chunk_px, &chunk_nx, &chunk_py, &chunk_ny,
            &chunk_pz, &chunk_nz,
        );

        // Create pipeline layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create the texture
        let size = 256u32;
        let texels = create_texels(size as usize);
        let texture_extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        queue.write_texture(
            texture.as_image_copy(),
            &texels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(size),
                rows_per_image: None,
            },
            texture_extent,
        );

        let min_alignment = device.limits().min_uniform_buffer_offset_alignment as usize;
        let uniform_size = std::mem::size_of::<Uniforms>();
        let aligned_uniform_size =
            ((uniform_size + min_alignment - 1) / min_alignment) * min_alignment;

        // Create other resources
        let uniform_vp_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform VP Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: aligned_uniform_size as wgpu::BufferAddress,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_vp_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
            label: None,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let vertex_size = mem::size_of::<Vertex>();

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 4,
                    shader_location: 1,
                },
            ],
        }];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(config.view_formats[0].into())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Done
        Vox {
            eye: glam::Vec3::new(0.0, -5.0, 3.0),
            horizontal_rotation: 0.0,
            vertical_rotation: 0.0,
            projection_matrix: Self::generate_projection_matrix(
                config.width as f32 / config.height as f32,
            ),
            depth_buffer,
            map,
            chunks: BTreeMap::new(),
            buffers: BTreeMap::new(),
            bind_group,
            uniform_vp_buffer,
            pipeline,
        }
    }

    fn update(&mut self, _event: winit::event::WindowEvent) {
        //empty
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        let texture_extent = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let draw_depth_buffer = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Buffer"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_buffer = draw_depth_buffer.create_view(&wgpu::TextureViewDescriptor::default());
        self.depth_buffer = depth_buffer;

        self.projection_matrix =
            Self::generate_projection_matrix(config.width as f32 / config.height as f32);
    }

    fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let dir = (glam::Mat3::from_rotation_z(self.horizontal_rotation)
                * glam::Mat3::from_rotation_x(self.vertical_rotation))
                * glam::Vec3::Y;
            let view_matrix = glam::Mat4::look_to_rh(self.eye, dir, glam::Vec3::Z);
            let mx_total = self.projection_matrix * view_matrix;
            let mx_ref: &[f32; 16] = mx_total.as_ref();
            queue.write_buffer(&self.uniform_vp_buffer, 0, bytemuck::cast_slice(mx_ref));
        }

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_buffer,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            for [x, y, z] in REGIONS {
                let buffers = self.get_buffers(device, x, y, z);
                rpass.push_debug_group("Prepare data for draw.");
                rpass.set_pipeline(&self.pipeline);
                rpass.set_index_buffer(buffers.1.slice(..), wgpu::IndexFormat::Uint16);
                rpass.set_vertex_buffer(0, buffers.0.slice(..));
                rpass.pop_debug_group();
                rpass.insert_debug_marker("Draw!");
                rpass.set_bind_group(0, &self.bind_group, &[]);
                rpass.draw_indexed(0..buffers.2, 0, 0..1);
            }
        }

        queue.submit(Some(encoder.finish()));
    }
}
