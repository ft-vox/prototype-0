use std::{cell::RefCell, marker::PhantomData, num::NonZeroU8, rc::Rc, sync::Arc};
use winit::{
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::{Key, NamedKey},
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod context;
mod input;
mod lru_cache;
mod map;
mod surface_wrapper;
mod texture;
mod vertex;
mod vox;
mod vox_update;

use context::Context;
use input::*;
use surface_wrapper::SurfaceWrapper;
use vox::*;

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
        builder = builder
            .with_title("ft_vox")
            .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720));
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
                vox.vertical_rotation = vox.vertical_rotation.clamp(
                    -0.4999 * std::f32::consts::PI,
                    0.4999 * std::f32::consts::PI,
                );
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
    let mut frame_driven_input = FrameDrivenInput::new();

    let event_loop_function = EventLoop::run;

    #[allow(clippy::let_unit_value)]
    let _ = (event_loop_function)(
        window_loop.event_loop,
        move |event: Event<()>, target: &EventLoopWindowTarget<()>| {
            match event {
                ref e if SurfaceWrapper::start_condition(e) => {
                    surface.resume(&context, window_loop.window.clone(), true);

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
                    } => event_driven_input.local_cursor_position = local_cursor_position,

                    WindowEvent::RedrawRequested => {
                        // On MacOS, currently redraw requested comes in _before_ Init does.
                        // If this happens, just drop the requested redraw on the floor.
                        //
                        // See https://github.com/rust-windowing/winit/issues/3235 for some discussion

                        frame_driven_input.update(&event_driven_input);

                        if let Some(vox) = vox.borrow_mut().as_mut() {
                            {
                                if let Ok(window_position) = window_loop.window.inner_position() {
                                    vox.update_window_info(
                                        window_position,
                                        window_loop.window.inner_size(),
                                    );
                                }
                                vox.update_eye_movement(&frame_driven_input);
                                vox.update_eye_rotation(&frame_driven_input);
                                vox.update_nearby_chunks(&context);
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
                    _ => {}
                },
                _ => {}
            }
        },
    );
}
