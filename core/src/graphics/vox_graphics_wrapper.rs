use std::sync::Arc;

use glam::{vec2, Vec3};

use ft_vox_prototype_0_map_types::CHUNK_SIZE;

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

        VoxGraphicsWrapper {
            world_renderer,
            sky_renderer,
            ui_renderer,
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

    pub fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        fog_distance: f32,
        buffers: Vec<MeshBuffer>,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.sky_renderer.render(queue, view, &mut encoder);
        self.world_renderer
            .render(queue, view, &mut encoder, buffers, fog_distance);

        let ui_item_bar = self.ui_renderer.create_ui_mesh(
            device,
            vec2(1600.0 / 2.0 - (182.0 * 4.0) / 2.0, 900.0 - (22.0 * 4.0)),
            vec2(182.0 * 4.0, 22.0 * 4.0),
            vec2(0.0, 0.0),
            vec2(182.0, 22.0),
            0,
        );

        let mut ui_active_item_highlight = self.ui_renderer.create_ui_mesh(
            device,
            vec2(1600.0 / 2.0 - (24.0 * 4.0) / 2.0, 900.0 - (23.0 * 4.0)),
            vec2(24.0 * 4.0, 24.0 * 4.0),
            vec2(0.0, 22.0),
            vec2(24.0, 24.0),
            0,
        );
        ui_active_item_highlight.1.position = vec2(0.0, 0.0);

        let ui_elements = vec![ui_item_bar, ui_active_item_highlight];
        self.ui_renderer
            .render(view, &mut encoder, queue, &ui_elements);

        queue.submit(Some(encoder.finish()));
    }
}
