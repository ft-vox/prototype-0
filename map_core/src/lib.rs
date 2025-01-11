use ft_vox_prototype_0_map_types::{
    Chunk, Cube, Harvestable, Plantlike, Solid, CHUNK_SIZE, MAP_HEIGHT,
};
use ft_vox_prototype_0_noise::{Noise, NoiseLayer};

const MIN_HEIGHT: f32 = 10.0;
const MAX_HEIGHT: f32 = 120.0;

#[derive(Clone)]
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

    // TODO: optimize
    pub fn get_chunk(&self, x: i32, y: i32) -> Chunk {
        let mut cubes = [Cube::Empty; MAP_HEIGHT * CHUNK_SIZE * CHUNK_SIZE];
        let x_offset = x * CHUNK_SIZE as i32;
        let y_offset = y * CHUNK_SIZE as i32;

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let actual_x = x_offset as f32 + x as f32;
                let actual_y = y_offset as f32 + y as f32;
                let noise = self.noise.noise2(actual_x * 0.042, actual_y * 0.042);
                let height = lerp(noise, MIN_HEIGHT, MAX_HEIGHT).floor() as usize;
                for z in 0..MAP_HEIGHT {
                    cubes[z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x] = if z == 0 {
                        Cube::Solid(Solid::Bedrock)
                    } else if height < z {
                        Cube::Empty
                    } else if height == z {
                        Cube::Harvestable(Harvestable::Wheat5)
                    } else if height == z + 1 {
                        Cube::Solid(Solid::GrassBlock)
                    } else if height == z + 2 {
                        Cube::Solid(Solid::Dirt)
                    } else {
                        Cube::Solid(Solid::Stone)
                    };
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
