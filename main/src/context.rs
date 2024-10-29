use ft_vox_prototype_0_core::MoveSpeed;
use ft_vox_prototype_0_core::TerrainWorker;
use ft_vox_prototype_0_core::Vox;
use winit::dpi::PhysicalPosition;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoopWindowTarget;

#[cfg(target_os = "windows")]
use winapi::um::winuser::SetCursorPos;

#[cfg(target_os = "macos")]
use core_graphics::{
    display::CGDisplay,
    event::{CGEvent, CGEventType, CGMouseButton},
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::CGPoint,
};

use crate::input::FrameDrivenInput;

pub struct Context<T: TerrainWorker> {
    pub vox: Vox<T>, // TODO: make it private
    window_inner_position: PhysicalPosition<i32>,
    window_inner_size: PhysicalSize<u32>,
    direction_and_speed: ([f32; 3], MoveSpeed), // TODO: separate
    pub horizontal_rotation: f32,
    pub vertical_rotation: f32,
}

impl<T: TerrainWorker> Context<T> {
    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        Context {
            vox: Vox::init(config, adapter, device, queue),
            window_inner_position: PhysicalPosition::new(0, 0),
            window_inner_size: PhysicalSize::new(0, 0),
            direction_and_speed: ([0.0, 0.0, 0.0], MoveSpeed::WALK),
            horizontal_rotation: 0.0,
            vertical_rotation: 0.0,
        }
    }

    pub fn update_window_info(
        &mut self,
        window_inner_position: PhysicalPosition<i32>,
        window_inner_size: PhysicalSize<u32>,
    ) {
        self.window_inner_position = window_inner_position;
        self.window_inner_size = window_inner_size;
    }

    pub fn update_eye_movement(&mut self, input: &FrameDrivenInput) {
        if self.vox.is_paused() {
            return;
        }

        let speed = if input.get_key_pressed("ctrl") {
            MoveSpeed::FLY
        } else {
            MoveSpeed::WALK
        };

        let direction = {
            let mut direction: [f32; 3] = [0.0, 0.0, 0.0];
            if input.get_key_pressed("w") {
                direction[1] += 1.0;
            }
            if input.get_key_pressed("a") {
                direction[0] -= 1.0;
            }
            if input.get_key_pressed("s") {
                direction[1] -= 1.0;
            }
            if input.get_key_pressed("d") {
                direction[0] += 1.0;
            }
            if input.get_key_pressed("space") {
                direction[2] += 1.0;
            }
            if input.get_key_pressed("shift") {
                direction[2] -= 1.0;
            }
            direction
        };

        self.direction_and_speed = (direction, speed);
    }

    pub fn update_eye_rotation(
        &mut self,
        input: &FrameDrivenInput,
        target: &EventLoopWindowTarget<()>,
    ) {
        if input.get_key_down("esc") {
            self.vox.set_is_paused(!self.vox.is_paused());
        }
        if self.vox.is_paused() {
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            const SENSITIVE: f32 = 0.0015;

            let window_position = self.window_inner_position;
            let window_size = self.window_inner_size;
            let delta_x = input.local_cursor_position.x as f32 - (window_size.width / 2) as f32;
            let delta_y = input.local_cursor_position.y as f32 - (window_size.height / 2) as f32;
            self.horizontal_rotation -= delta_x * SENSITIVE;
            self.vertical_rotation -= delta_y * SENSITIVE;

            let center_x: i32 = window_position.x + (window_size.width / 2) as i32;
            let center_y: i32 = window_position.y + (window_size.height / 2) as i32;

            #[cfg(target_os = "windows")]
            unsafe {
                SetCursorPos(center_x, center_y);
            }

            #[cfg(target_os = "macos")]
            {
                let display_size_os = target.primary_monitor().unwrap().size();
                let display_size_cg = CGDisplay::main().bounds().size;
                let scaling_factor = display_size_cg.width / display_size_os.width as f64;
                let scaled_x = center_x as f64 * scaling_factor;
                let scaled_y = center_y as f64 * scaling_factor;
                let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap();
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

    pub fn tick(&mut self, delta_time: f32) {
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

    pub fn set_mouse_center(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let window_position = self.window_inner_position;
            let window_size = self.window_inner_size;

            let center_x: i32 = window_position.x + (window_size.width / 2) as i32;
            let center_y: i32 = window_position.y + (window_size.height / 2) as i32;

            #[cfg(target_os = "windows")]
            unsafe {
                SetCursorPos(center_x, center_y);
            }

            #[cfg(target_os = "macos")]
            {
                let display_size_os = target.primary_monitor().unwrap().size();
                let display_size_cg = CGDisplay::main().bounds().size;
                let scaling_factor = display_size_cg.width / display_size_os.width as f64;
                let scaled_x = center_x as f64 * scaling_factor;
                let scaled_y = center_y as f64 * scaling_factor;
                let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap();
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
}
