use crate::context::Context;
use crate::input::*;
use crate::map::*;
use crate::vox::*;
use winit::dpi::PhysicalPosition;
use winit::dpi::PhysicalSize;

#[cfg(target_os = "windows")]
use winapi::um::winuser::SetCursorPos;

#[cfg(target_os = "macos")]
use core_graphics::{
    display::CGDisplay,
    event::{CGEvent, CGEventType, CGMouseButton},
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::CGPoint,
};

/// [ Speed in Minecraft ]
/// Walking speed: 4.317 blocks/second
/// Sprinting speed (Survival): 5.612 blocks/second
/// Flying speed (Creative): 10.89 blocks/second

const MOVE_SPEED: f32 = 4.317;
const FAST_MOVE_SPEED: f32 = 10.89;

const FPS: f32 = 60.0;

impl Vox {
    pub fn update_window_info(
        &mut self,
        window_inner_position: PhysicalPosition<i32>,
        window_inner_size: PhysicalSize<u32>,
    ) {
        self.window_inner_position = window_inner_position;
        self.window_inner_size = window_inner_size;
    }

    pub fn update_eye_movement(&mut self, input: &FrameDrivenInput) {
        let speed = if input.get_key_pressed("ctrl") {
            FAST_MOVE_SPEED / FPS
        } else {
            MOVE_SPEED / FPS
        };
        if input.get_key_pressed("w") && !input.get_key_pressed("s") {
            let forward_x = -self.horizontal_rotation.sin();
            let forward_y = self.horizontal_rotation.cos();
            self.eye.x += forward_x * speed;
            self.eye.y += forward_y * speed;
        }

        if input.get_key_pressed("a") && !input.get_key_pressed("d") {
            let forward_x = -self.horizontal_rotation.sin();
            let forward_y = self.horizontal_rotation.cos();
            let leftward_x = -forward_y;
            let leftward_y = forward_x;
            self.eye.x += leftward_x * speed;
            self.eye.y += leftward_y * speed;
        }

        if input.get_key_pressed("s") && !input.get_key_pressed("w") {
            let forward_x = -self.horizontal_rotation.sin();
            let forward_y = self.horizontal_rotation.cos();
            self.eye.x -= forward_x * speed;
            self.eye.y -= forward_y * speed;
        }

        if input.get_key_pressed("d") && !input.get_key_pressed("a") {
            let forward_x = -self.horizontal_rotation.sin();
            let forward_y = self.horizontal_rotation.cos();
            let rightward_x = forward_y;
            let rightward_y = -forward_x;
            self.eye.x += rightward_x * speed;
            self.eye.y += rightward_y * speed;
        }

        if input.get_key_pressed("space") && !input.get_key_pressed("shift") {
            self.eye.z += 0.1;
        }

        if input.get_key_pressed("shift") && !input.get_key_pressed("str") {
            self.eye.z -= 0.1;
        }
    }

    pub fn update_eye_rotation(&mut self, input: &FrameDrivenInput) {
        if input.get_key_down("tab") {
            self.mouse_lock = !self.mouse_lock;
        }
        if !self.mouse_lock {
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let sensitive: f32 = 0.0015;
            let window_position = self.window_inner_position;
            let window_size = self.window_inner_size;
            let delta_x = input.local_cursor_position.x - (window_size.width / 2) as f64;
            let delta_y = input.local_cursor_position.y - (window_size.height / 2) as f64;
            self.horizontal_rotation -= delta_x as f32 * sensitive;
            self.horizontal_rotation %= 2.0 * std::f32::consts::PI;
            if self.horizontal_rotation < 0.0 {
                self.horizontal_rotation += 2.0 * std::f32::consts::PI;
            }

            self.vertical_rotation -= delta_y as f32 * sensitive;
            self.vertical_rotation = self.vertical_rotation.clamp(
                -0.4999 * std::f32::consts::PI,
                0.4999 * std::f32::consts::PI,
            );

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

    pub fn update_nearby_chunks(&mut self, context: &Context) {
        let coords = Self::get_coords(RENDER_DISTANCE);
        let where_am_i = self.eye.floor() / CHUNK_SIZE as f32;
        for coord in coords.iter() {
            self.get_buffers(
                &context.device,
                coord.0 + where_am_i.x as i32,
                coord.1 + where_am_i.y as i32,
                coord.2 + where_am_i.z as i32,
            );
        }
    }
}
