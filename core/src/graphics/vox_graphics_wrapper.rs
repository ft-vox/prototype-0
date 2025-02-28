use std::sync::Arc;
use std::time::Instant;

use glam::{vec2, Vec3};

use map_types::CHUNK_SIZE;

use crate::graphics::font_info::FontInfo;
use crate::graphics::{SkyRenderer, UIRenderer, WorldRenderer};
use crate::FOV;
use crate::RENDER_DISTANCE;

pub type DrawCallArgs = (wgpu::Buffer, wgpu::Buffer, u32);

pub struct MeshBuffer {
    pub x: i32,
    pub y: i32,
    pub opaque: Arc<Vec<DrawCallArgs>>,
    pub translucent: Arc<Vec<DrawCallArgs>>,
}

pub struct VoxGraphicsWrapper {
    world_renderer: WorldRenderer,
    sky_renderer: SkyRenderer,
    ui_renderer: UIRenderer,
    font_info: FontInfo,
    ui_elements: Vec<(
        crate::graphics::ui_renderer::UIMeshWGPU,
        crate::graphics::ui_renderer::UITransform,
    )>,
    text_meshes: Vec<(
        crate::graphics::ui_renderer::UIMeshWGPU,
        crate::graphics::ui_renderer::UITransform,
    )>,
    last_frame_time: Instant,
    current_fps: u32,
}

impl VoxGraphicsWrapper {
    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let world_renderer = WorldRenderer::init(
            config,
            _adapter,
            device,
            queue,
            FOV,
            0.25,
            CHUNK_SIZE as f32 * RENDER_DISTANCE,
        );

        let sky_renderer = SkyRenderer::init(config, _adapter, device, queue, FOV, 0.25, 1000.0);
        let ui_renderer = UIRenderer::init(config, device, queue);

        let ui_item_bar = ui_renderer.create_ui_mesh(
            device,
            vec2(1600.0 / 2.0 - (182.0 * 4.0) / 2.0, 900.0 - (22.0 * 4.0)),
            vec2(182.0 * 4.0, 22.0 * 4.0),
            vec2(0.0, 0.0),
            vec2(182.0, 22.0),
            0,
        );
        let ui_elements = vec![ui_item_bar];

        let font_info = FontInfo::new(1, 16.0, 16.0);
        let text_meshes = ui_renderer.create_text_mesh(
            device,
            "FPS 0\nTriangle 0",
            vec2(20.0, 20.0),
            1.0,
            &font_info,
        );

        VoxGraphicsWrapper {
            world_renderer,
            sky_renderer,
            ui_renderer,
            font_info,
            ui_elements,
            text_meshes,
            last_frame_time: Instant::now(),
            current_fps: 0,
        }
    }

    pub fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.sky_renderer.resize(config);
        self.world_renderer.resize(config, device);
        self.ui_renderer.resize(config);
    }

    pub fn update(&mut self, eye: Vec3, eye_dir: Vec3) {
        self.sky_renderer.update(eye_dir);
        self.world_renderer.update(eye, eye_dir);
    }

    pub fn update_text(&mut self, device: &wgpu::Device, text: &str) {
        self.text_meshes =
            self.ui_renderer
                .create_text_mesh(device, text, vec2(20.0, 20.0), 1.0, &self.font_info);
    }

    pub fn update_info_text(&mut self, device: &wgpu::Device, fps: u32, triangle_count: u32) {
        let info_text = format!("FPS {}\nTriangle {}", fps, triangle_count);
        self.update_text(device, &info_text);
    }

    fn calculate_fps(&mut self) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        let frame_time_secs = frame_time.as_secs_f64();
        let new_fps = if frame_time_secs > 0.0 {
            (1.0 / frame_time_secs).round() as u32
        } else {
            0
        };
        self.current_fps = new_fps;
    }

    pub fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        fog_distance: f32,
        buffers: Vec<MeshBuffer>,
    ) {
        self.calculate_fps();

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.sky_renderer.render(queue, view, &mut encoder);
        self.world_renderer
            .render(queue, view, &mut encoder, buffers, fog_distance);

        let triangle_count = self.world_renderer.get_triangle_count();
        self.update_info_text(device, self.current_fps, triangle_count);

        let mut ui_element_refs = Vec::new();
        for element in &self.ui_elements {
            ui_element_refs.push(element);
        }
        for element in &self.text_meshes {
            ui_element_refs.push(element);
        }
        self.ui_renderer
            .render(view, &mut encoder, queue, &ui_element_refs);

        queue.submit(Some(encoder.finish()));
    }
}
