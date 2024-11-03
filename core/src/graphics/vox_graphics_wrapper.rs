use std::sync::Arc;

use glam::Vec3;

use ft_vox_prototype_0_map_types::CHUNK_SIZE;

use crate::graphics::{SkyRenderer, WorldRenderer};
use crate::FOV;
use crate::RENDER_DISTANCE;

pub struct VoxGraphicsWrapper {
    world_renderer: WorldRenderer,
    sky_renderer: SkyRenderer,
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

        VoxGraphicsWrapper {
            world_renderer,
            sky_renderer,
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
        buffer_data: Vec<((i32, i32, i32), Arc<(wgpu::Buffer, wgpu::Buffer, u32)>)>,
    ) {
        let mut encoder: wgpu::CommandEncoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.sky_renderer.render(queue, view, &mut encoder);
        self.world_renderer
            .render(queue, view, &mut encoder, buffer_data, fog_distance);

        queue.submit(Some(encoder.finish()));
    }
}
