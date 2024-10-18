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
