pub const CHUNK_SIZE: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cube {
    Empty,
    Solid(Solid),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Solid {
    Grass,
    Dirt,
    Stone,
}

#[derive(Clone)]
pub struct Chunk {
    pub cubes: [Cube; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
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

    pub fn tex_coord_px(&self) -> [[f32; 2]; 4] {
        match self {
            Cube::Solid(Solid::Grass) => [[4.0, 1.0], [3.0, 1.0], [3.0, 0.0], [4.0, 0.0]],
            _ => unreachable!("Incorrect cube type"),
        }
    }

    pub fn tex_coord_nx(&self) -> [[f32; 2]; 4] {
        match self {
            Cube::Solid(Solid::Grass) => [[4.0, 0.0], [3.0, 0.0], [3.0, 1.0], [4.0, 1.0]],
            _ => unreachable!("Incorrect cube type"),
        }
    }

    pub fn tex_coord_py(&self) -> [[f32; 2]; 4] {
        match self {
            Cube::Solid(Solid::Grass) => [[3.0, 1.0], [4.0, 1.0], [4.0, 0.0], [3.0, 0.0]],
            _ => unreachable!("Incorrect cube type"),
        }
    }

    pub fn tex_coord_ny(&self) -> [[f32; 2]; 4] {
        match self {
            Cube::Solid(Solid::Grass) => [[3.0, 0.0], [4.0, 0.0], [4.0, 1.0], [3.0, 1.0]],
            _ => unreachable!("Incorrect cube type"),
        }
    }

    pub fn tex_coord_pz(&self) -> [[f32; 2]; 4] {
        match self {
            Cube::Solid(Solid::Grass) => [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            _ => unreachable!("Incorrect cube type"),
        }
    }

    pub fn tex_coord_nz(&self) -> [[f32; 2]; 4] {
        match self {
            Cube::Solid(Solid::Grass) => [[3.0, 0.0], [2.0, 0.0], [2.0, 1.0], [3.0, 1.0]],
            _ => unreachable!("Incorrect cube type"),
        }
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
        let mut cubes = [Cube::Empty; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        for i in 0..CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE {
            cubes[i] = Cube::from_u8(from[i]);
        }
        Self { cubes }
    }
}
