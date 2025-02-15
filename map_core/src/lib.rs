use map_types::{
    Chunk, Cube, Custom, FilteredSolid, Plantlike, Solid, Translucent, CHUNK_SIZE, MAP_HEIGHT,
};
use noise::{Noise, NoiseLayer};

pub const WATER_LEVEL: usize = 111;

#[derive(Clone)]
pub struct Map {
    main_noise: Noise,
    height_base_noise: Noise,
}

impl Map {
    pub fn new(seed: u64) -> Map {
        let main_noise = Noise::new(
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
        let height_base_noise = Noise::new(
            &[
                NoiseLayer::new(0.00142, 0.9),
                NoiseLayer::new(0.0042, 0.07),
                NoiseLayer::new(0.042, 0.03),
            ],
            seed,
        );
        Map {
            main_noise,
            height_base_noise,
        }
    }

    // TODO: optimize
    pub fn get_chunk(&self, x: i32, y: i32) -> Chunk {
        let mut cubes = [Cube::Empty; MAP_HEIGHT * CHUNK_SIZE * CHUNK_SIZE];
        let mut biome_colors = [[0.0f32; 4]; CHUNK_SIZE * CHUNK_SIZE];
        let x_offset = x * CHUNK_SIZE as i32;
        let y_offset = y * CHUNK_SIZE as i32;

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let actual_x = x_offset as f32 + x as f32;
                let actual_y = y_offset as f32 + y as f32;
                macro_rules! n {
                    ($factor:expr, $z:expr) => {
                        self.main_noise
                            .noise3(actual_x * $factor, actual_y * $factor, $z)
                    };
                }

                let humidity = n!(0.0042, 4242.0);
                let biome1 = n!(0.042, 4242.0);
                let biome2 = n!(0.042, 42042.0);
                let biome3 = n!(0.042, 424242.0);
                let biome4 = n!(0.042, 420420.0);
                let is_sand = humidity < -0.2;
                let height = (lerp(
                    self.height_base_noise.noise2(actual_x, actual_y) / 4.0 + 0.5,
                    22.2,
                    222.2,
                ) + (n!(0.0618, 0.0) * n!(0.000922, 42.0)) * 342.0)
                    .clamp(22.2, 222.2) as usize;
                biome_colors[y * CHUNK_SIZE + x] = [
                    (de_lerp(biome1, -0.1, 0.1).sin() / 2.0 + 0.5).powi(2),
                    (de_lerp(biome2, -0.1, 0.1).sin() / 2.0 + 0.5).powi(2),
                    (de_lerp(biome3, -0.1, 0.1).sin() / 2.0 + 0.5).powi(2),
                    (de_lerp(biome4, -0.1, 0.1).sin() / 2.0 + 0.5).powi(8),
                ];
                for z in 0..MAP_HEIGHT {
                    cubes[z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x] = if z == 0 {
                        Cube::Solid(Solid::Bedrock)
                    } else if height + 1 == z
                        && is_sand
                        && z > WATER_LEVEL
                        && n!(1.0, 420.0) > 0.1949
                        && n!(1.0, 402.0) <= -0.2
                    {
                        Cube::Custom(Custom::Cactus)
                    } else if height < z {
                        if z <= WATER_LEVEL {
                            Cube::Translucent(Translucent::Ice) // TODO: water
                        } else {
                            Cube::Empty
                        }
                    } else if height == z {
                        if z <= WATER_LEVEL {
                            Cube::Translucent(Translucent::Ice) // TODO: water
                        } else if is_sand && n!(1.0, 420.0) > 0.1949 {
                            let n = n!(1.0, 402.0);
                            if n > -0.2 {
                                Cube::Plantlike(Plantlike::DeadBush)
                            } else {
                                Cube::Custom(Custom::Cactus)
                            }
                        } else if !is_sand && n!(1.0, 420.0) > 0.2042 {
                            let n = n!(1.0, 402.0);
                            if n > 0.2 {
                                Cube::Plantlike(Plantlike::FlowerRed)
                            } else if n < -0.2 {
                                Cube::Plantlike(Plantlike::FlowerYellow)
                            } else {
                                Cube::Plantlike(Plantlike::Grass)
                            }
                        } else {
                            Cube::Empty
                        }
                    } else if height == z + 1 {
                        if !is_sand && height == WATER_LEVEL {
                            Cube::Solid(Solid::Dirt)
                        } else if !is_sand && height < WATER_LEVEL {
                            Cube::Solid(Solid::Gravel)
                        } else if is_sand {
                            Cube::Solid(Solid::Sand)
                        } else {
                            Cube::FilteredSolid(FilteredSolid::GrassBlock)
                        }
                    } else if height == z + 2 {
                        if !is_sand && height >= WATER_LEVEL {
                            Cube::Solid(Solid::Dirt)
                        } else if !is_sand {
                            Cube::Solid(Solid::Gravel)
                        } else {
                            Cube::Solid(Solid::Sand)
                        }
                    } else {
                        Cube::Solid(Solid::Stone)
                    };
                }
            }
        }

        Chunk {
            cubes,
            biome_colors,
        }
    }
}

fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

fn de_lerp(result: f32, a: f32, b: f32) -> f32 {
    (result - a) / (b - a)
}
