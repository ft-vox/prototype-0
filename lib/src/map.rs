use ft_vox_prototype_0_noise::{Noise, NoiseLayer};

pub const CHUNK_SIZE: usize = 13;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cube {
    Empty,
    OnlyOneAtThisTime,
}

#[derive(Clone)]
pub struct Chunk {
    pub cubes: [Cube; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
}

pub struct Map {
    noise: Noise,
}

impl Map {
    pub fn new(seed: u64) -> Map {
        let noise = Noise::new(
            &[
                NoiseLayer::new(0.05, 0.1),
                NoiseLayer::new(0.1, 0.1),
                NoiseLayer::new(0.15, 0.2),
                NoiseLayer::new(0.2, 0.2),
                NoiseLayer::new(0.6, 0.2),
                NoiseLayer::new(1.2, 0.2),
            ],
            seed,
        );

        Map { noise }
    }

    pub fn get_chunk(&self, x: i32, y: i32, z: i32) -> Chunk {
        let mut cubes = [Cube::Empty; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        let x_offset = x * CHUNK_SIZE as i32;
        let y_offset = y * CHUNK_SIZE as i32;
        let z_offset = z * CHUNK_SIZE as i32;

        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    const MIN_HEIGHT: f32 = -10.0;
                    const MAX_HEIGHT: f32 = 10.0;
                    let actual_x = x_offset as f32 + x as f32;
                    let actual_y = y_offset as f32 + y as f32;
                    let actual_z = z_offset as f32 + z as f32;
                    let noise = self.noise.noise3(actual_x, actual_y, actual_z);
                    let density = lerp(
                        de_lerp(actual_z, MIN_HEIGHT, MAX_HEIGHT).clamp(0.0, 1.0),
                        1.0,
                        -1.0,
                    ) + noise;
                    let cube = if density > 0.0 {
                        Cube::OnlyOneAtThisTime
                    } else {
                        Cube::Empty
                    };
                    cubes[z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x] = cube;
                }
            }
        }

        Chunk { cubes }
    }
}

fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

fn de_lerp(result: f32, a: f32, b: f32) -> f32 {
    (result - a) / (b - a)
}
