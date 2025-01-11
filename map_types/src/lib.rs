pub const CHUNK_SIZE: usize = 16;
pub const MAP_HEIGHT: usize = 128;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cube {
    Empty,
    Solid(Solid),
    Translucent(Translucent),
    Plantlike(Plantlike),
    Harvestable(Harvestable),
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
    // 0
    Bedrock((1, 1)),
    GrassBlock((0, 3), (0, 0), (0, 2)),
    Dirt((0, 2)),
    Stone((0, 1)),
    PlankOak((0, 4)), // TODO: rename
    PlankBirch((13, 6)), // TODO: rename
    PlankJungle((12, 7)), // TODO: rename
    PlankSpruce((12, 6)), // TODO: rename
    SmoothStone((0, 6)),
    SmoothStoneSlabs((0, 5), (0, 6), (0, 6)),
    Bricks((0, 7)),
    TNT((0, 8), (0, 9), (0, 10)),
    // 1
    Cobblestone((1, 0)),
    Sand((1, 2)),
    Gravel((1, 3)),
    OakLog((1, 4), (1, 5), (1, 5)),
    BlockOfIron((1, 6)),
    BlockOfGold((1, 7)),
    BlockOfDiamond((1, 8)),
    // 2
    GoldOre((2, 0)),
    IronOre((2, 1)),
    CoalOre((2, 2)),
    Bookshelf((2, 3), (0, 4), (0, 4)),
    MossyCobblestone((2, 4)),
    // 3
    Obsidian((2, 5)),
    Sponge((3, 0)),
    DiamondOre((3, 2)),
    RedstoneOre((3, 3)),
    StoneBricks((3, 5)),
    // 4
    WoolWhite((4, 0)),
    SnowBlock((4, 2)),
    SnowyGrassBlock((4, 4), (4, 2), (0, 2)),
    Clay((4, 8)),
    Jukebox((4, 10), (4, 11), (4, 10)),
    Mycelium((4, 13), (4, 14), (0, 2)),



}

macro_rules! define_translucent {
    ($($variant:ident($($vals:tt),*)),* $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub enum Translucent {
            $($variant),*
        }

        impl Translucent {
            pub fn tex_coord(&self) -> [[f32; 2]; 4] {
                match self {
                    $(
                        Self::$variant => {
                            define_translucent!(@tex_coord $($vals),*)
                        }
                    ),*
                }
            }
        }
    };

    (@tex_coord $y:expr, $x:expr) => {
        [[($x + 1) as f32, ($y + 1) as f32], [$x as f32, ($y + 1) as f32], [$x as f32, $y as f32], [($x + 1) as f32, $y as f32]]
    };
}

define_translucent! {
    Glass(3, 1),
    OakLeaves(3, 4),
    MonsterSpawner(4, 1),
    Ice(4, 3),
}

macro_rules! define_plantlike {
    ($($variant:ident($($vals:tt),*)),* $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub enum Plantlike {
            $($variant),*
        }

        impl Plantlike {
            pub fn tex_coord(&self) -> [[f32; 2]; 4] {
                match self {
                    $(
                        Self::$variant => {
                            define_plantlike!(@tex_coord $($vals),*)
                        }
                    ),*
                }
            }
        }
    };

    (@tex_coord $y:expr, $x:expr) => {
        [[($x + 1) as f32, ($y + 1) as f32], [$x as f32, ($y + 1) as f32], [$x as f32, $y as f32], [($x + 1) as f32, $y as f32]]
    };
}

define_plantlike! {
    Grass(2, 7),
    FlowerRed(0, 12),
    FlowerYellow(0, 13),
    MushroomRed(1, 12),
    MushroomBrown(1, 13),
    TreeSamplingOak(0, 15),
    TreeSamplingBirch(4, 15),
    TreeSamplingJungle(1, 14),
    TreeSamplingSpruce(3, 15),
    TreeSamplingLikeIDK(3, 8),
    DeadBush(3, 7),
    Cobweb(0, 11),
}

macro_rules! define_harvestable {
    ($($variant:ident($($vals:tt),*)),* $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub enum Harvestable {
            $($variant),*
        }

        impl Harvestable {
            pub fn tex_coord(&self) -> [[f32; 2]; 4] {
                match self {
                    $(
                        Self::$variant => {
                            define_harvestable!(@tex_coord $($vals),*)
                        }
                    ),*
                }
            }
        }
    };

    (@tex_coord $y:expr, $x:expr) => {
        [[($x + 1) as f32, ($y + 1) as f32], [$x as f32, ($y + 1) as f32], [$x as f32, $y as f32], [($x + 1) as f32, $y as f32]]
    };
}

define_harvestable! {
    Wheat1(5, 8),
    Wheat2(5, 9),
    Wheat3(5, 10),
    Wheat4(5, 11),
    Wheat5(5, 12),
    Wheat6(5, 13),
    Wheat7(5, 14),
    Wheat8(5, 15),
    NetherWart1(14, 2),
    NetherWart2(14, 3),
    NetherWart3(14, 4),
}

#[derive(Clone)]
pub struct Chunk {
    pub cubes: [Cube; MAP_HEIGHT * CHUNK_SIZE * CHUNK_SIZE],
}

impl Cube {
    pub fn is_solid(&self) -> bool {
        matches!(self, Cube::Solid(_))
    }

    pub fn is_translucent_or_solid(&self) -> bool {
        matches!(self, Cube::Translucent(_) | Cube::Solid(_))
    }
}
