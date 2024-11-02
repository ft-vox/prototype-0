use std::{cell::RefCell, num::NonZeroU8, rc::Rc, sync::Arc};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::Key,
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod context;
mod input;
mod surface_wrapper;
mod wgpu_context;

use context::Context;
use input::*;
use surface_wrapper::SurfaceWrapper;
use wgpu_context::WGPUContext;

use ft_vox_prototype_0_core::TerrainWorker;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

struct EventLoopWrapper {
    event_loop: EventLoop<()>,
    window: Arc<Window>,
    #[cfg(target_arch = "wasm32")]
    canvas: Option<web_sys::Element>,
    #[cfg(not(target_arch = "wasm32"))]
    canvas: Option<NonZeroU8>,
}

impl EventLoopWrapper {
    pub fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        let mut builder = winit::window::WindowBuilder::new();
        builder = builder
            .with_title("ft_vox")
            .with_position(PhysicalPosition::new(100, 100))
            .with_inner_size(PhysicalSize::new(1280, 720))
            .with_min_inner_size(winit::dpi::LogicalSize::new(160.0, 90.0));
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

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
async fn run_in_wasm() {
    run::<ft_vox_prototype_0_terrain_worker_web::WebTerrainWorker>().await;
}

pub async fn run<T: TerrainWorker + 'static>() {
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
    let wgpu_context = WGPUContext::init_async(&mut surface, window_loop.window.clone()).await;
    let context: Rc<RefCell<Option<Context<T>>>> = Rc::new(RefCell::new(None));
    #[cfg(target_arch = "wasm32")]
    {
        const SENSITIVE: f32 = 0.0015;

        let vox = Rc::clone(&context);

        let context = context.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if let Some(context) = context.borrow_mut().as_mut() {
                let delta_x = event.movement_x() as f32;
                let delta_y = event.movement_y() as f32;

                context.horizontal_rotation -= delta_x * SENSITIVE;
                context.vertical_rotation -= delta_y * SENSITIVE;
            }
        }) as Box<dyn FnMut(_)>);
        window_loop
            .canvas
            .unwrap()
            .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())
            .expect("Failed to add mousemove event listener");
        closure.forget();
    }

    let mut event_driven_input = EventDrivenInput::new();

    let event_loop_function = EventLoop::run;

    #[cfg(not(target_arch = "wasm32"))]
    let mut last_frame_time = Instant::now();
    #[cfg(target_arch = "wasm32")]
    fn performance_now() -> f32 {
        web_sys::window().unwrap().performance().unwrap().now() as f32
    }
    #[cfg(target_arch = "wasm32")]
    let mut last_frame_time = performance_now();

    #[allow(clippy::let_unit_value)]
    let _ = (event_loop_function)(
        window_loop.event_loop,
        move |event: Event<()>, target: &EventLoopWindowTarget<()>| {
            match event {
                ref e if SurfaceWrapper::start_condition(e) => {
                    surface.resume(&wgpu_context, window_loop.window.clone(), true);

                    // If we haven't created the example yet, do so now.
                    if context.borrow().is_none() {
                        *context.borrow_mut() = Some(Context::init(
                            surface.config(),
                            &wgpu_context.adapter,
                            &wgpu_context.device,
                            &wgpu_context.queue,
                            window_loop.window.clone(),
                        ));
                        if let Some(context) = context.borrow_mut().as_mut() {
                            //context.set_mouse_center(target);
                        }
                    }
                }

                Event::Suspended => {
                    surface.suspend();
                }

                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        if let Some(context) = context.borrow_mut().as_mut() {
                            context.resize(size, &mut surface, &wgpu_context);
                        }
                        window_loop.window.request_redraw();
                    }

                    WindowEvent::CloseRequested => {
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
                        println!("{:#?}", wgpu_context.instance.generate_report());
                    }

                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key, state, ..
                            },
                        ..
                    } => event_driven_input.set_key_state(logical_key, state),

                    WindowEvent::CursorMoved {
                        position: local_cursor_position,
                        ..
                    } => event_driven_input.set_cursor_position(local_cursor_position),

                    WindowEvent::RedrawRequested => {
                        // On MacOS, currently redraw requested comes in _before_ Init does.
                        // If this happens, just drop the requested redraw on the floor.
                        //
                        // See https://github.com/rust-windowing/winit/issues/3235 for some discussion

                        #[cfg(not(target_arch = "wasm32"))]
                        let delta_time = {
                            let now = Instant::now();
                            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
                            last_frame_time = now;
                            delta_time
                        };
                        #[cfg(target_arch = "wasm32")]
                        let delta_time = {
                            let now = performance_now();
                            let delta_time = now - last_frame_time;
                            last_frame_time = now;
                            delta_time / 1000.0
                        };

                        if let Some(context) = context.borrow_mut().as_mut() {
                            context.update(&event_driven_input);
                            context.tick(delta_time);
                            context.render(&mut surface, &wgpu_context);

                            window_loop.window.request_redraw();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        },
    );
}
