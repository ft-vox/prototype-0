use std::sync::Arc;

use tokio::net::tcp::OwnedWriteHalf;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    window::{Fullscreen, Window},
};

use game_core::player::MoveSpeed;
use game_core::Vox;
use messages::ServerMessage; // ★ ServerMessage 임포트 (Debug 가능해졌음)

use crate::surface_wrapper::SurfaceWrapper;
use crate::{
    input::{EventDrivenInput, FrameDrivenInput},
    wgpu_context::WGPUContext,
};

pub struct Context {
    vox: Vox,
    window: Arc<Window>,
    input: FrameDrivenInput,
    window_inner_position: PhysicalPosition<i32>,
    window_inner_size: PhysicalSize<u32>,
    direction_and_speed: ([f32; 3], MoveSpeed), // TODO: separate
    pub horizontal_rotation: f32,
    pub vertical_rotation: f32,

    fly_toggle: bool,
    fly_toggle_timer: Option<f32>,

    adhoc_winit_fault_cursor_position_x: f64,
    adhoc_winit_fault_cursor_position_y: f64,
}

impl Context {
    /// init: OwnedWriteHalf를 받아 Vox::init(...) 호출
    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: Arc<Window>,
        write_half: OwnedWriteHalf,
    ) -> Self {
        println!("\n[ CONTROL KEYS ]\nmovement: WASD + Shift + Space\nspeeding: CTRL\npause: ESC\nscreen mode: Tab");

        Context {
            vox: Vox::init(config, adapter, device, queue, write_half),
            window,
            input: FrameDrivenInput::new(),
            window_inner_position: PhysicalPosition::new(0, 0),
            window_inner_size: PhysicalSize::new(0, 0),
            direction_and_speed: ([0.0, 0.0, 0.0], MoveSpeed::Walk),
            horizontal_rotation: 0.0,
            vertical_rotation: 0.0,
            fly_toggle: false,
            fly_toggle_timer: None,
            adhoc_winit_fault_cursor_position_x: 0.0,
            adhoc_winit_fault_cursor_position_y: 0.0,
        }
    }

    pub fn resize(
        &mut self,
        size: PhysicalSize<u32>,
        surface: &mut SurfaceWrapper,
        wgpu_context: &WGPUContext,
    ) {
        surface.resize(wgpu_context, size);
        self.vox
            .resize(surface.config(), &wgpu_context.device, &wgpu_context.queue);
    }

    pub fn update(&mut self, event_driven_input: &EventDrivenInput) {
        self.update_input(event_driven_input);
        self.update_window_info();
        self.update_eye_movement();
        self.update_eye_rotation();

        self.update_screen_mode();
    }

    pub fn tick(&mut self, delta_time: f32) {
        self.update_mouse_lock(delta_time);
        if let Some(ref mut fly_toggle_timer) = self.fly_toggle_timer {
            if *fly_toggle_timer > 0.3 {
                self.fly_toggle_timer = None;
            } else if self.input.get_key_down("space") {
                self.fly_toggle_timer = None;
                self.fly_toggle = !self.fly_toggle;
            } else {
                *fly_toggle_timer += delta_time;
            }
        }
        if self.input.get_key_down("space") {
            self.fly_toggle_timer = Some(0.0);
        }

        self.vox.tick(
            delta_time,
            self.direction_and_speed.0,
            self.direction_and_speed.1,
            self.horizontal_rotation,
            self.vertical_rotation,
        );
        self.direction_and_speed = ([0.0, 0.0, 0.0], self.direction_and_speed.1);
        self.horizontal_rotation = 0.0;
        self.vertical_rotation = 0.0;
    }

    pub fn render(&mut self, surface: &mut SurfaceWrapper, wgpu_context: &WGPUContext) {
        let frame = surface.acquire(wgpu_context);
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(surface.config().view_formats[0]),
            ..wgpu::TextureViewDescriptor::default()
        });
        self.vox
            .render(&view, &wgpu_context.device, &wgpu_context.queue);
        frame.present();
    }

    /// ★ ServerMessage 처리 메서드
    /// 여기서 debug 출력이 가능해짐: `{:?}`
    pub fn handle_server_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::Init { your_player_id, your_position } => {
                println!("Init => pid={}, position={:?}", your_player_id, your_position);
            }
            ServerMessage::PlayerMove { moved_player_id, position } => {
                println!("PlayerMove => pid={}, position={:?}", moved_player_id, position);
            }
            other => {
                println!("Unhandled message: {:?}", other);
            }
        }
    }

    ////////////////////////////
    ///////// Private //////////
    ////////////////////////////

    fn update_input(&mut self, event_driven_input: &EventDrivenInput) {
        self.input.update(event_driven_input);
    }

    fn update_window_info(&mut self) {
        if let Ok(window_inner_position) = self.window.inner_position() {
            self.window_inner_position = window_inner_position;
        }
        self.window_inner_size = self.window.inner_size();
    }

    fn update_eye_movement(&mut self) {
        if self.vox.is_paused() {
            return;
        }

        let speed = if self.fly_toggle {
            MoveSpeed::CreativeFly
        } else {
            MoveSpeed::Walk
        };

        let direction = {
            let mut direction: [f32; 3] = [0.0, 0.0, 0.0];
            if self.input.get_key_pressed("w") {
                direction[1] += 1.0;
            }
            if self.input.get_key_pressed("a") {
                direction[0] -= 1.0;
            }
            if self.input.get_key_pressed("s") {
                direction[1] -= 1.0;
            }
            if self.input.get_key_pressed("d") {
                direction[0] += 1.0;
            }
            if self.input.get_key_pressed("space") {
                direction[2] += 1.0;
            }
            if self.input.get_key_pressed("shift") {
                direction[2] -= 1.0;
            }
            direction
        };

        self.direction_and_speed = (direction, speed);
    }

    fn update_eye_rotation(&mut self) {
        if self.input.get_key_down("esc") {
            self.vox.set_is_paused(!self.vox.is_paused());
        }
        if self.vox.is_paused() {
            return;
        }

        const SENSITIVE: f32 = 0.001;

        let window_size = self.window_inner_size;
        let center_x = window_size.width / 2;
        let center_y = window_size.height / 2;

        let (x, y) = if self.input.local_cursor_position.x
            == self.adhoc_winit_fault_cursor_position_x
            && self.input.local_cursor_position.y == self.adhoc_winit_fault_cursor_position_y
        {
            (center_x as f64, center_y as f64)
        } else {
            (
                self.input.local_cursor_position.x,
                self.input.local_cursor_position.y,
            )
        };
        self.adhoc_winit_fault_cursor_position_x = self.input.local_cursor_position.x;
        self.adhoc_winit_fault_cursor_position_y = self.input.local_cursor_position.y;
        let delta_x = x - center_x as f64;
        let delta_y = y - center_y as f64;
        self.horizontal_rotation -= delta_x as f32 * SENSITIVE;
        self.vertical_rotation -= delta_y as f32 * SENSITIVE;

        self.window
            .set_cursor_position(PhysicalPosition::new(center_x, center_y))
            .unwrap();
        self.update_window_info();
    }

    fn update_mouse_lock(&mut self, delta_time: f32) {
        let fps = (1.0 / delta_time).round() as i32;
        if self.vox.is_paused() {
            self.window.set_cursor_visible(true);
            self.window.set_title("ft_vox: paused");
        } else {
            self.window.set_cursor_visible(false);
            self.window.set_title(&format!("ft_vox: {} fps", fps));
        }
    }

    fn update_screen_mode(&mut self) {
        if self.input.get_key_down("tab") {
            if self.window.fullscreen().is_some() {
                self.window.set_fullscreen(None);
            } else {
                self.window
                    .set_fullscreen(Some(Fullscreen::Borderless(None)));
            }
        }
    }
}
