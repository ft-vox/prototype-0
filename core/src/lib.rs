use ft_vox_prototype_0_map_types::{Chunk, CHUNK_SIZE};
use glam::{Mat3, Vec3};
use std::sync::{Arc, Mutex};
use terrain_manager::TerrainManager;
use wgpu::util::DeviceExt;

pub mod terrain_manager;
pub mod vertex;
mod vox_graphics_wrapper;

use vertex::Vertex;
use vox_graphics_wrapper::*;

pub const CACHE_DISTANCE: usize = 19;
pub const RENDER_DISTANCE: f32 = CACHE_DISTANCE as f32;

pub const FOG_COLOR: [f32; 4] = [57.0 / 255.0, 107.0 / 255.0, 251.0 / 255.0, 1.0];
pub const FOG_END: f32 = (RENDER_DISTANCE - 2.0) * CHUNK_SIZE as f32;
pub const FOG_START: f32 = FOG_END * 0.8;

pub const FOV: f32 = 120.0;

pub enum TerrainWorkerJob {
    Map((i32, i32, i32)),
    Mesh {
        position: (i32, i32, i32),
        zero: Arc<Chunk>,
        positive_x: Arc<Chunk>,
        negative_x: Arc<Chunk>,
        positive_y: Arc<Chunk>,
        negative_y: Arc<Chunk>,
        positive_z: Arc<Chunk>,
        negative_z: Arc<Chunk>,
    },
}

pub trait TerrainWorker {
    fn new(
        job_callback: Arc<Mutex<dyn Send + Sync + FnMut() -> Option<TerrainWorkerJob>>>,
        chunk_callback: Arc<Mutex<dyn Send + Sync + FnMut((i32, i32, i32), Arc<Chunk>)>>,
        mesh_callback: Arc<
            Mutex<dyn Send + Sync + FnMut((i32, i32, i32), (Vec<Vertex>, Vec<u16>))>,
        >,
    ) -> Self;
}

pub fn get_coords(distance: f32) -> Vec<(i32, i32, i32)> {
    let mut coords = Vec::new();
    let max_coord = distance.floor() as i32;
    let distance_squared = distance * distance;

    for x in -max_coord..=max_coord {
        for y in -max_coord..=max_coord {
            for z in -max_coord..=max_coord {
                let dist_sq = (x * x + y * y + z * z) as f32;
                if dist_sq <= distance_squared {
                    coords.push((x, y, z));
                }
            }
        }
    }

    coords
}

pub struct Vox<T: TerrainWorker> {
    vox_graphics_wrapper: VoxGraphicsWrapper,
    eye: glam::Vec3,
    horizontal_rotation: f32,
    vertical_rotation: f32,
    is_paused: bool,
    chunk_cache: TerrainManager<T, Arc<(wgpu::Buffer, wgpu::Buffer, u32)>>,
}

/// [ Speed in Minecraft ]
/// Walking speed: 4.317 blocks/second
/// Sprinting speed (Survival): 5.612 blocks/second
/// Flying speed (Creative): 10.89 blocks/second

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MoveSpeed {
    Walk,
    SubjectFly,
    CreativeFly,
}

impl MoveSpeed {
    pub const fn speed_per_sec(&self) -> f32 {
        match self {
            Self::Walk => 4.317,
            Self::SubjectFly => 20.00,
            Self::CreativeFly => 10.89,
        }
    }
}

impl<T: TerrainWorker> Vox<T> {
    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let vox_graphics_wrapper = VoxGraphicsWrapper::init(config, _adapter, device, queue);
        let eye_x = 0.0;
        let eye_y = -5.0;
        let eye_z = 3.0;

        // Done
        Vox {
            vox_graphics_wrapper,
            eye: glam::Vec3::new(eye_x, eye_y, eye_z),
            horizontal_rotation: 0.0,
            vertical_rotation: 0.0,
            is_paused: false,
            chunk_cache: TerrainManager::new(CACHE_DISTANCE, (eye_x, eye_y, eye_z)),
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
        move_speed: MoveSpeed,
        delta_horizontal_rotation: f32,
        delta_vertical_rotation: f32,
    ) {
        if self.is_paused {
            return;
        }

        // rotate
        {
            self.horizontal_rotation += delta_horizontal_rotation;
            self.horizontal_rotation %= 2.0 * std::f32::consts::PI;
            if self.horizontal_rotation < 0.0 {
                self.horizontal_rotation += 2.0 * std::f32::consts::PI;
            }

            self.vertical_rotation += delta_vertical_rotation;
            self.vertical_rotation = self.vertical_rotation.clamp(
                -0.4999 * std::f32::consts::PI,
                0.4999 * std::f32::consts::PI,
            );
        }

        // move
        {
            let move_direction = {
                let move_direction = Mat3::from_rotation_z(self.horizontal_rotation)
                    * Vec3::new(move_direction[0], move_direction[1], move_direction[2]);
                let move_speed = move_direction.length();
                if move_speed > 1.0 {
                    move_direction / move_speed
                } else {
                    move_direction
                }
            };
            self.eye += move_direction * move_speed.speed_per_sec() * delta_time;
        }
    }

    pub fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.chunk_cache.set_cache_distance(CACHE_DISTANCE);
        self.chunk_cache
            .set_eye((self.eye.x, self.eye.y, self.eye.z));
        let res = self.chunk_cache.get_available(&mut |vertices, indices| {
            Arc::new((
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                }),
                indices.len() as u32,
            ))
        });

        self.vox_graphics_wrapper.render(
            view,
            device,
            queue,
            self.eye,
            self.horizontal_rotation,
            self.vertical_rotation,
            res,
        );
    }
}
