pub const CHUNK_SIZE: usize = 16;
pub const MAP_HEIGHT: usize = 128;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cube {
    Empty,
    Solid(Solid),
    // PlantLike(PlantLike),
}

macro_rules! define_solid {
    ($($variant:ident($($vals:tt),*)),* $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub enum Solid {
            $($variant),*
        }

        impl Solid {
            pub fn tex_coord_px(&self) -> [[f32; 2]; 4] {
                match self {
                    $(
                        Self::$variant => {
                            define_solid!(@tex_coord_px $($vals),*)
                        }
                    ),*
                }
            }
            pub fn tex_coord_nx(&self) -> [[f32; 2]; 4] {
                match self {
                    $(
                        Self::$variant => {
                            define_solid!(@tex_coord_nx $($vals),*)
                        }
                    ),*
                }
            }
            pub fn tex_coord_py(&self) -> [[f32; 2]; 4] {
                match self {
                    $(
                        Self::$variant => {
                            define_solid!(@tex_coord_py $($vals),*)
                        }
                    ),*
                }
            }
            pub fn tex_coord_ny(&self) -> [[f32; 2]; 4] {
                match self {
                    $(
                        Self::$variant => {
                            define_solid!(@tex_coord_ny $($vals),*)
                        }
                    ),*
                }
            }
            pub fn tex_coord_pz(&self) -> [[f32; 2]; 4] {
                match self {
                    $(
                        Self::$variant => {
                            define_solid!(@tex_coord_pz $($vals),*)
                        }
                    ),*
                }
            }
            pub fn tex_coord_nz(&self) -> [[f32; 2]; 4] {
                match self {
                    $(
                        Self::$variant => {
                            define_solid!(@tex_coord_nz $($vals),*)
                        }
                    ),*
                }
            }
        }
    };

    (@tex_coord_px ($y:expr, $x:expr)) => {
        define_solid!(@tex_coord_px ($y, $x), ($y, $x), ($y, $x))
    };
    (@tex_coord_px ($side_y:expr, $side_x:expr), ($top_y:expr, $top_x:expr), ($bottom_y:expr, $bottom_x:expr)) => {
        [[($side_x + 1) as f32, ($side_y + 1) as f32], [$side_x as f32, ($side_y + 1) as f32], [$side_x as f32, $side_y as f32], [($side_x + 1) as f32, $side_y as f32]]
    };
    (@tex_coord_nx ($y:expr, $x:expr)) => {
        define_solid!(@tex_coord_nx ($y, $x), ($y, $x), ($y, $x))
    };
    (@tex_coord_nx ($side_y:expr, $side_x:expr), ($top_y:expr, $top_x:expr), ($bottom_y:expr, $bottom_x:expr)) => {
        [[($side_x + 1) as f32, $side_y as f32], [$side_x as f32, $side_y as f32], [$side_x as f32, ($side_y + 1) as f32], [($side_x + 1) as f32, ($side_y + 1) as f32]]
    };
    (@tex_coord_py ($y:expr, $x:expr)) => {
        define_solid!(@tex_coord_py ($y, $x), ($y, $x), ($y, $x))
    };
    (@tex_coord_py ($side_y:expr, $side_x:expr), ($top_y:expr, $top_x:expr), ($bottom_y:expr, $bottom_x:expr)) => {
        [[$side_x as f32, ($side_y + 1) as f32], [($side_x + 1) as f32, ($side_y + 1) as f32], [($side_x + 1) as f32, $side_y as f32], [$side_x as f32, $side_y as f32]]
    };
    (@tex_coord_ny ($y:expr, $x:expr)) => {
        define_solid!(@tex_coord_ny ($y, $x), ($y, $x), ($y, $x))
    };
    (@tex_coord_ny ($side_y:expr, $side_x:expr), ($top_y:expr, $top_x:expr), ($bottom_y:expr, $bottom_x:expr)) => {
        [[$side_x as f32, $side_y as f32], [($side_x + 1) as f32, $side_y as f32], [($side_x + 1) as f32, ($side_y + 1) as f32], [$side_x as f32, ($side_y + 1) as f32]]
    };
    (@tex_coord_pz ($y:expr, $x:expr)) => {
        define_solid!(@tex_coord_pz ($y, $x), ($y, $x), ($y, $x))
    };
    (@tex_coord_pz ($side_y:expr, $side_x:expr), ($top_y:expr, $top_x:expr), ($bottom_y:expr, $bottom_x:expr)) => {
        [[($top_x + 1) as f32, ($top_y + 1) as f32], [$top_x as f32, ($top_y + 1) as f32], [$top_x as f32, $top_y as f32], [($top_x + 1) as f32, $top_y as f32]]
    };
    (@tex_coord_nz ($y:expr, $x:expr)) => {
        define_solid!(@tex_coord_nz ($y, $x), ($y, $x), ($y, $x))
    };
    (@tex_coord_nz ($side_y:expr, $side_x:expr), ($top_y:expr, $top_x:expr), ($bottom_y:expr, $bottom_x:expr)) => {
        [[($bottom_x + 1) as f32, ($bottom_y + 1) as f32], [$bottom_x as f32, ($bottom_y + 1) as f32], [$bottom_x as f32, $bottom_y as f32], [($bottom_x + 1) as f32, $bottom_y as f32]]
    };
}

define_solid! {
    Grass((0, 3), (0, 0), (0, 2)),
    Dirt((0, 2)),
    Stone((0, 1)),
}

// #[derive(Clone, Copy, PartialEq, Eq)]
// pub enum PlantLike {
//     Grass,
//     FlowerRed,
//     FlowerYellow,
//     MushroomRed,
//     MushroomBrown,
//     TreeSamplingOak,
//     TreeSamplingBirch,
//     TreeSamplingJungle,
//     TreeSamplingSpruce,
//     TreeSamplingLikeIDK,
// }

#[derive(Clone)]
pub struct Chunk {
    pub cubes: [Cube; MAP_HEIGHT * CHUNK_SIZE * CHUNK_SIZE],
}

impl Cube {
    pub fn to_u8(&self) -> u8 {
        match self {
            Cube::Empty => 0,
            Cube::Solid(solid) => solid.to_u8(),
        }
    }

    pub fn from_u8(u8: u8) -> Self {
        match u8 {
            0 => Cube::Empty,
            1 => Cube::Solid(Solid::Grass),
            2 => Cube::Solid(Solid::Dirt),
            3 => Cube::Solid(Solid::Stone),
            _ => panic!("Invalid cube given"),
        }
    }

    pub fn is_solid(&self) -> bool {
        matches!(self, Cube::Solid(_))
    }
}

impl Solid {
    pub fn to_u8(&self) -> u8 {
        match self {
            Solid::Grass => 1,
            Solid::Dirt => 2,
            Solid::Stone => 3,
        }
    }
}

impl Chunk {
    pub fn to_u8_vec(&self) -> Vec<u8> {
        self.cubes.iter().map(Cube::to_u8).collect()
    }

    pub fn from_u8_vec(from: &[u8]) -> Self {
        let mut cubes = [Cube::Empty; MAP_HEIGHT * CHUNK_SIZE * CHUNK_SIZE];
        for i in 0..MAP_HEIGHT * CHUNK_SIZE * CHUNK_SIZE {
            cubes[i] = Cube::from_u8(from[i]);
        }
        Self { cubes }
    }
}
