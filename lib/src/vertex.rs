use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}

pub fn vertex(pos: [f32; 3], tc: [f32; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0], pos[1], pos[2], 1.0],
        _tex_coord: [tc[0], tc[1]],
    }
}

pub fn create_vertices(x: f32, y: f32, z: f32, index: usize) -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        // top (0, 0, 1)
        vertex([x + 0.0, y + 0.0, z + 1.0], [0.0, 0.0]),
        vertex([x + 1.0, y + 0.0, z + 1.0], [1.0, 0.0]),
        vertex([x + 1.0, y + 1.0, z + 1.0], [1.0, 1.0]),
        vertex([x + 0.0, y + 1.0, z + 1.0], [0.0, 1.0]),
        // bottom (0, 0, -1)
        vertex([x + 0.0, y + 1.0, z + 0.0], [1.0, 0.0]),
        vertex([x + 1.0, y + 1.0, z + 0.0], [0.0, 0.0]),
        vertex([x + 1.0, y + 0.0, z + 0.0], [0.0, 1.0]),
        vertex([x + 0.0, y + 0.0, z + 0.0], [1.0, 1.0]),
        // right (1, 0, 0)
        vertex([x + 1.0, y + 0.0, z + 0.0], [0.0, 0.0]),
        vertex([x + 1.0, y + 1.0, z + 0.0], [1.0, 0.0]),
        vertex([x + 1.0, y + 1.0, z + 1.0], [1.0, 1.0]),
        vertex([x + 1.0, y + 0.0, z + 1.0], [0.0, 1.0]),
        // left (-1, 0, 0)
        vertex([x + 0.0, y + 0.0, z + 1.0], [1.0, 0.0]),
        vertex([x + 0.0, y + 1.0, z + 1.0], [0.0, 0.0]),
        vertex([x + 0.0, y + 1.0, z + 0.0], [0.0, 1.0]),
        vertex([x + 0.0, y + 0.0, z + 0.0], [1.0, 1.0]),
        // front (0, 1, 0)
        vertex([x + 1.0, y + 1.0, z + 0.0], [1.0, 0.0]),
        vertex([x + 0.0, y + 1.0, z + 0.0], [0.0, 0.0]),
        vertex([x + 0.0, y + 1.0, z + 1.0], [0.0, 1.0]),
        vertex([x + 1.0, y + 1.0, z + 1.0], [1.0, 1.0]),
        // back (0, -1, 0)
        vertex([x + 1.0, y + 0.0, z + 1.0], [0.0, 0.0]),
        vertex([x + 0.0, y + 0.0, z + 1.0], [1.0, 0.0]),
        vertex([x + 0.0, y + 0.0, z + 0.0], [1.0, 1.0]),
        vertex([x + 1.0, y + 0.0, z + 0.0], [0.0, 1.0]),
    ];

    let offset = index as u16 * 24;
    let index_data: &[u16] = &[
        offset,
        1 + offset,
        2 + offset,
        2 + offset,
        3 + offset,
        offset, // top
        4 + offset,
        5 + offset,
        6 + offset,
        6 + offset,
        7 + offset,
        4 + offset, // bottom
        8 + offset,
        9 + offset,
        10 + offset,
        10 + offset,
        11 + offset,
        8 + offset, // right
        12 + offset,
        13 + offset,
        14 + offset,
        14 + offset,
        15 + offset,
        12 + offset, // left
        16 + offset,
        17 + offset,
        18 + offset,
        18 + offset,
        19 + offset,
        16 + offset, // front
        20 + offset,
        21 + offset,
        22 + offset,
        22 + offset,
        23 + offset,
        20 + offset, // back
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

pub fn create_texels(size: usize) -> Vec<u8> {
    (0..size * size)
        .map(|id| {
            // get high five for recognizing this ;)
            let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
            let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
            let (mut x, mut y, mut count) = (cx, cy, 0);
            while count < 0xFF && x * x + y * y < 4.0 {
                let old_x = x;
                x = x * x - y * y + cx;
                y = 2.0 * old_x * y + cy;
                count += 1;
            }
            count
        })
        .collect()
}
