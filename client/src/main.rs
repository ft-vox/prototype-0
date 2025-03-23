use std::{cell::RefCell, env, process::exit, rc::Rc, sync::Arc, time::Instant};

use tokio::net::{tcp::OwnedReadHalf, TcpStream};
use tokio::sync::mpsc;
use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::Window,
};

mod context;
mod input;
mod surface_wrapper;
mod wgpu_context;

use crate::context::Context;
use crate::input::EventDrivenInput;
use crate::surface_wrapper::SurfaceWrapper;
use crate::wgpu_context::WGPUContext;
use messages::ServerMessage;

struct EventLoopWrapper {
    event_loop: EventLoop<()>,
    window: Arc<Window>,
}

impl EventLoopWrapper {
    pub fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(
            winit::window::WindowBuilder::new()
                .with_title("ft_vox")
                .with_inner_size(PhysicalSize::new(1280, 720))
                .build(&event_loop)
                .unwrap(),
        );
        Self { event_loop, window }
    }
}

/// 서버 메시지 수신 태스크 (읽기 전용)
async fn network_listener(mut read_half: OwnedReadHalf, tx: mpsc::Sender<ServerMessage>) {
    use tokio::io::AsyncReadExt;
    let mut buffer = vec![0u8; 1024];
    loop {
        match read_half.read(&mut buffer).await {
            Ok(0) => {
                println!("Server disconnected.");
                break;
            }
            Ok(n) => {
                if let Ok(msg) = bincode::deserialize::<ServerMessage>(&buffer[..n]) {
                    if tx.send(msg).await.is_err() {
                        eprintln!("Failed to send server message to channel");
                    }
                }
            }
            Err(e) => {
                eprintln!("Read error: {:?}", e);
                break;
            }
        }
    }
    println!("network_listener finished.");
}

#[tokio::main]
async fn main() {
    let server_addr = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: cargo run --release --bin client <ip>:<port>");
        exit(0)
    });

    // 1) 서버 연결
    let stream = TcpStream::connect(&server_addr).await.unwrap();
    // 2) split -> read_half, write_half
    let (read_half, write_half) = stream.into_split();
    // write_half를 Option으로 감싸서 한 번만 move하도록 함
    let write_half_opt = Rc::new(RefCell::new(Some(write_half)));

    // 3) 백그라운드에서 read_half → network_listener
    let (tx, mut rx) = mpsc::channel::<ServerMessage>(100);
    tokio::spawn(network_listener(read_half, tx));

    // 4) winit & wgpu
    env_logger::init();
    let window_loop = EventLoopWrapper::new();
    let mut surface = SurfaceWrapper::new();
    let wgpu_context = WGPUContext::init_async(&mut surface, window_loop.window.clone()).await;

    // 5) Context 생성은 나중에
    let context: Rc<RefCell<Option<Context>>> = Rc::new(RefCell::new(None));
    let mut event_input = EventDrivenInput::new();
    let mut last_frame_time = Instant::now();

    // 6) 이벤트 루프
    #[allow(clippy::let_unit_value)]
    let _ = window_loop.event_loop.run(
        move |event: Event<()>, target: &EventLoopWindowTarget<()>| {
            match event {
                ref e if SurfaceWrapper::start_condition(e) => {
                    surface.resume(&wgpu_context, window_loop.window.clone(), true);
                    if context.borrow().is_none() {
                        // 7) Context::init(..., write_half) 한 번만 move하기 위해 Option에서 take()
                        if let Some(wh) = write_half_opt.borrow_mut().take() {
                            *context.borrow_mut() = Some(Context::init(
                                surface.config(),
                                &wgpu_context.adapter,
                                &wgpu_context.device,
                                &wgpu_context.queue,
                                window_loop.window.clone(),
                                wh,
                            ));
                        } else {
                            eprintln!("write_half already taken!");
                        }
                    }
                }
                Event::Suspended => {
                    surface.suspend();
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        if let Some(ctx) = context.borrow_mut().as_mut() {
                            ctx.resize(size, &mut surface, &wgpu_context);
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
                    } => {
                        event_input.set_key_state(logical_key, state);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        event_input.set_cursor_position(position);
                    }
                    WindowEvent::RedrawRequested => {
                        let now = Instant::now();
                        let delta_time = now.duration_since(last_frame_time).as_secs_f32();
                        last_frame_time = now;

                        while let Ok(msg) = rx.try_recv() {
                            if let Some(ctx) = context.borrow_mut().as_mut() {
                                ctx.handle_server_message(msg);
                            }
                        }

                        if let Some(ctx) = context.borrow_mut().as_mut() {
                            ctx.update(&event_input);
                            ctx.tick(delta_time);
                            ctx.render(&mut surface, &wgpu_context);
                        }
                        window_loop.window.request_redraw();
                    }
                    _ => {}
                },
                _ => {}
            }
        },
    );
}
