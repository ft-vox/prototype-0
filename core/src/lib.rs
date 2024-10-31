use std::sync::{Arc, Mutex};

use glam::{Mat3, Vec3};
use wgpu::util::DeviceExt;

use ft_vox_prototype_0_map_types::Chunk;

pub mod terrain_manager;
pub mod vertex;
mod vox_graphics_wrapper;

use terrain_manager::TerrainManager;
use vertex::Vertex;
use vox_graphics_wrapper::VoxGraphicsWrapper;

pub const CACHE_DISTANCE: usize = 12;
pub const RENDER_DISTANCE: f32 = CACHE_DISTANCE as f32;

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
    eye: Vec3,
    horizontal_rotation: f32,
    vertical_rotation: f32,
    eye_dir: Vec3,
    is_paused: bool,
    terrain_manager: TerrainManager<T, Arc<(wgpu::Buffer, wgpu::Buffer, u32)>>,
    target_fog_distance: f32,
    current_fog_distance: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MoveSpeed {
    Walk,
    Sprint,
    CreativeFly,
    FtVoxFly,
    FtMinecraftFly,
}

impl MoveSpeed {
    pub const fn speed_per_sec(&self) -> f32 {
        match self {
            Self::Walk => 4.317,
            Self::Sprint => 5.612,
            Self::CreativeFly => 10.89,
            Self::FtVoxFly => 20.00,
            Self::FtMinecraftFly => 40.00,
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
            eye_dir: Vec3::new(0.0, 0.0, 0.0),
            is_paused: false,
            terrain_manager: TerrainManager::new(CACHE_DISTANCE, (eye_x, eye_y, eye_z)),

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
        move_speed: MoveSpeed,
        delta_horizontal_rotation: f32,
        delta_vertical_rotation: f32,
    ) {
        if self.is_paused {
            return;
        }

        // eye_dir
        {
            self.eye_dir = (glam::Mat3::from_rotation_z(self.horizontal_rotation)
                * glam::Mat3::from_rotation_x(self.vertical_rotation))
                * glam::Vec3::Y;
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
        self.terrain_manager
            .set_eye((self.eye.x, self.eye.y, self.eye.z));

        let wgpu_buffers = self
            .terrain_manager
            .get_available(&mut |vertices, indices| {
                Arc::new((
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
                ))
            });

        self.vox_graphics_wrapper.render(
            view,
            device,
            queue,
            self.eye,
            self.eye_dir,
            self.current_fog_distance,
            wgpu_buffers,
        );
    }
}
