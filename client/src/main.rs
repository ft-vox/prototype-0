use std::env;
use std::process::exit;
use std::time::Instant;
use std::{cell::RefCell, rc::Rc, sync::Arc};
use tokio::net::TcpStream;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::Window,
};

mod context;
mod input;
mod surface_wrapper;
mod wgpu_context;

use crate::context::Context;
use crate::input::*;
use crate::surface_wrapper::SurfaceWrapper;
use crate::wgpu_context::WGPUContext;

struct EventLoopWrapper {
    event_loop: EventLoop<()>,
    window: Arc<Window>,
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

        Self { event_loop, window }
    }
}

#[tokio::main]
async fn main() {
    let server_addr = env::args().nth(1).unwrap_or_else(|| {
        println!("Usage: cargo run --release --bin client <server ip>:<port>");
        exit(0)
    });
    let mut stream = Some(TcpStream::connect(&server_addr).await.unwrap());

    env_logger::init();

    let window_loop = EventLoopWrapper::new();
    let mut surface = SurfaceWrapper::new();
    let wgpu_context = WGPUContext::init_async(&mut surface, window_loop.window.clone()).await;
    let context: Rc<RefCell<Option<Context>>> = Rc::new(RefCell::new(None));

    let mut event_driven_input = EventDrivenInput::new();

    let event_loop_function = EventLoop::run;

    let mut last_frame_time = Instant::now();

    #[allow(clippy::let_unit_value)]
    let _ = (event_loop_function)(
        window_loop.event_loop,
        move |event: Event<()>, target: &EventLoopWindowTarget<()>| {
            match event {
                ref e if SurfaceWrapper::start_condition(e) => {
                    surface.resume(&wgpu_context, window_loop.window.clone(), true);

                    if context.borrow().is_none() {
                        *context.borrow_mut() = Some(Context::init(
                            surface.config(),
                            &wgpu_context.adapter,
                            &wgpu_context.device,
                            &wgpu_context.queue,
                            window_loop.window.clone(),
                            stream.take().unwrap(),
                        ));
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

                        let delta_time = {
                            let now = Instant::now();
                            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
                            last_frame_time = now;
                            delta_time
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
