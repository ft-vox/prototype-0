use std::sync::Arc;

use glam::{Mat3, Vec3};
use wgpu::util::DeviceExt;

mod graphics;
pub mod player;
pub mod terrain_manager;
mod terrain_worker;
pub mod vertex;

use graphics::VoxGraphicsWrapper;
use player::Human;
use terrain_manager::TerrainManager;

pub const CACHE_DISTANCE: usize = 30;
pub const RENDER_DISTANCE: f32 = CACHE_DISTANCE as f32;
pub const FOG_COLOR_SRGB: [f32; 4] = [130.0 / 255.0, 173.0 / 255.0, 253.0 / 255.0, 1.0];
pub const FOV: f32 = 80.0;

pub fn get_coords(distance: f32) -> Vec<(i32, i32)> {
    let mut coords = Vec::new();
    let max_coord = distance.floor() as i32;
    let distance_squared = distance * distance;

    for x in -max_coord..=max_coord {
        for y in -max_coord..=max_coord {
            let dist_sq = (x * x + y * y) as f32;
            if dist_sq <= distance_squared {
                coords.push((x, y));
            }
        }
    }

    coords
}

pub struct Vox {
    vox_graphics_wrapper: VoxGraphicsWrapper,
    local_player: Human,
    is_paused: bool,
    terrain_manager: TerrainManager,
    target_fog_distance: f32,
    current_fog_distance: f32,
}

impl Vox {
    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let vox_graphics_wrapper = VoxGraphicsWrapper::init(config, _adapter, device, queue);
        let eye_x = 0.0;
        let eye_y = -5.0;
        let eye_z = 120.0;

        // Done
        Vox {
            vox_graphics_wrapper,
            local_player: Human::new(Vec3::new(eye_x, eye_y, eye_z)),
            is_paused: false,
            terrain_manager: TerrainManager::new(CACHE_DISTANCE, (eye_x, eye_y)),
            target_fog_distance: 0.0,
            current_fog_distance: 0.0,
        }
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn set_is_paused(&mut self, paused: bool) {
        self.is_paused = paused;
    }

    pub fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.vox_graphics_wrapper.resize(config, device, _queue);
    }

    pub fn tick(
        &mut self,
        delta_time: f32,
        move_direction: [f32; 3],
        move_speed: player::MoveSpeed,
        delta_horizontal_rotation: f32,
        delta_vertical_rotation: f32,
    ) {
        if self.is_paused {
            return;
        }

        self.local_player.move_speed = move_speed;
        self.local_player.update(
            delta_time,
            move_direction,
            delta_horizontal_rotation,
            delta_vertical_rotation,
        );

        // smooth fog distance
        {
            self.target_fog_distance = self.terrain_manager.get_farthest_distance().max(6.0);

            let step = 3.5 * delta_time;
            if self.target_fog_distance != self.current_fog_distance {
                let direction = (self.target_fog_distance - self.current_fog_distance).signum();
                self.current_fog_distance += step * direction;
                if (self.current_fog_distance - self.target_fog_distance) * direction > 0.0 {
                    self.current_fog_distance = self.target_fog_distance;
                }
            }
        }
    }

    pub fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.terrain_manager.set_cache_distance(CACHE_DISTANCE);
        let eye_pos = self.local_player.get_eye_position();
        self.terrain_manager.set_eye((eye_pos.x, eye_pos.y));

        let buffers = self.terrain_manager.get_available(&mut |mesh| {
            (
                Arc::new(
                    mesh.opaque_buffers
                        .iter()
                        .map(|(vertices, indices)| {
                            (
                                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: Some("Vertex Buffer"),
                                    contents: bytemuck::cast_slice(vertices),
                                    usage: wgpu::BufferUsages::VERTEX,
                                }),
                                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: Some("Index Buffer"),
                                    contents: bytemuck::cast_slice(indices),
                                    usage: wgpu::BufferUsages::INDEX,
                                }),
                                indices.len() as u32,
                            )
                        })
                        .collect(),
                ),
                Arc::new(
                    mesh.translucent_buffers
                        .iter()
                        .map(|(vertices, indices)| {
                            (
                                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: Some("Vertex Buffer"),
                                    contents: bytemuck::cast_slice(vertices),
                                    usage: wgpu::BufferUsages::VERTEX,
                                }),
                                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: Some("Index Buffer"),
                                    contents: bytemuck::cast_slice(indices),
                                    usage: wgpu::BufferUsages::INDEX,
                                }),
                                indices.len() as u32,
                            )
                        })
                        .collect(),
                ),
            )
        });

        self.vox_graphics_wrapper.update(
            self.local_player.get_eye_position(),
            self.local_player.get_eye_direction(),
        );
        self.vox_graphics_wrapper
            .render(view, device, queue, self.current_fog_distance, buffers);
    }
}
