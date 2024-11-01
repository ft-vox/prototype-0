use std::sync::Arc;

use winit::event_loop::EventLoopWindowTarget;

#[cfg(target_os = "macos")]
use core_graphics::{
    display::CGDisplay,
    event::{CGEvent, CGEventType, CGMouseButton},
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::CGPoint,
};
#[cfg(target_os = "windows")]
use winapi::um::winuser::SetCursorPos;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    window::{Fullscreen, Window},
};

use ft_vox_prototype_0_core::MoveSpeed;
use ft_vox_prototype_0_core::TerrainWorker;
use ft_vox_prototype_0_core::Vox;

use crate::surface_wrapper::SurfaceWrapper;
use crate::{
    input::{EventDrivenInput, FrameDrivenInput},
    wgpu_context::WGPUContext,
};

pub struct Context<T: TerrainWorker> {
    vox: Vox<T>,
    window: Arc<Window>,
    input: FrameDrivenInput,
    window_inner_position: PhysicalPosition<i32>,
    window_inner_size: PhysicalSize<u32>,
    direction_and_speed: ([f32; 3], MoveSpeed), // TODO: separate
    horizontal_rotation: f32,
    vertical_rotation: f32,

    fly_toggle: bool,
    fly_toggle_timer: Option<f32>,
}

impl<T: TerrainWorker> Context<T> {
    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: Arc<Window>,
    ) -> Self {
        println!("\n[ CONTROL KEYS ]\nmovement: WASD + Shift + Space\nspeeding: CTRL\npause: ESC\nscreen mode: Tab");
        Context {
            vox: Vox::init(config, adapter, device, queue),
            window,
            input: FrameDrivenInput::new(),
            window_inner_position: PhysicalPosition::new(0, 0),
            window_inner_size: PhysicalSize::new(0, 0),
            direction_and_speed: ([0.0, 0.0, 0.0], MoveSpeed::Walk),
            horizontal_rotation: 0.0,
            vertical_rotation: 0.0,
            fly_toggle: false,
            fly_toggle_timer: None,
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
        self.update_mouse_lock();
        self.update_screen_mode();
    }

    pub fn tick(&mut self, delta_time: f32) {
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
            MoveSpeed::FtVoxFly
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

        #[cfg(not(target_arch = "wasm32"))]
        {
            const SENSITIVE: f32 = 0.0015;

            let window_position = self.window_inner_position;
            let window_size = self.window_inner_size;
            let delta_x =
                self.input.local_cursor_position.x as f32 - (window_size.width / 2) as f32;
            let delta_y =
                self.input.local_cursor_position.y as f32 - (window_size.height / 2) as f32;
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
                let display_size_os = self.window.primary_monitor().unwrap().size();
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

    fn update_mouse_lock(&mut self) {
        if self.vox.is_paused() {
            self.window.set_cursor_visible(true);
            self.window.set_title("ft_vox: paused");
        } else {
            self.window.set_cursor_visible(false);
            self.window.set_title("ft_vox");
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

    pub fn set_mouse_center(&mut self, target: &EventLoopWindowTarget<()>) {
        self.update_window_info();
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
