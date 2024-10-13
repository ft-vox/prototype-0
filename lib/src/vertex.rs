use bytemuck::{Pod, Zeroable};

use crate::map::*;

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

pub fn create_vertices_for_chunk(
    chunk: &Chunk,
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    chunk_px: &Chunk,
    chunk_nx: &Chunk,
    chunk_py: &Chunk,
    chunk_ny: &Chunk,
    chunk_pz: &Chunk,
    chunk_nz: &Chunk,
) -> (Vec<Vertex>, Vec<u16>) {
    let x_offset = chunk_x * CHUNK_SIZE as i32;
    let y_offset = chunk_y * CHUNK_SIZE as i32;
    let z_offset = chunk_z * CHUNK_SIZE as i32;

    let mut vertex_data = Vec::<Vertex>::new();
    let mut index_data = Vec::<u16>::new();
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if chunk.cubes[z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x].is_solid() {
                    let actual_x = x_offset + x as i32;
                    let actual_y = y_offset + y as i32;
                    let actual_z = z_offset + z as i32;
                    let (mut tmp_vertex_data, mut tmp_index_data) = create_vertices(
                        actual_x as f32,
                        actual_y as f32,
                        actual_z as f32,
                        if x == CHUNK_SIZE - 1 {
                            chunk_px.cubes[z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE].is_solid()
                        } else {
                            chunk.cubes[z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x + 1]
                                .is_solid()
                        },
                        if x == 0 {
                            chunk_nx.cubes
                                [z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + CHUNK_SIZE - 1]
                                .is_solid()
                        } else {
                            chunk.cubes[z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x - 1]
                                .is_solid()
                        },
                        if y == CHUNK_SIZE - 1 {
                            chunk_py.cubes[z * CHUNK_SIZE * CHUNK_SIZE + x].is_solid()
                        } else {
                            chunk.cubes[z * CHUNK_SIZE * CHUNK_SIZE + (y + 1) * CHUNK_SIZE + x]
                                .is_solid()
                        },
                        if y == 0 {
                            chunk_ny.cubes
                                [z * CHUNK_SIZE * CHUNK_SIZE + (CHUNK_SIZE - 1) * CHUNK_SIZE + x]
                                .is_solid()
                        } else {
                            chunk.cubes[z * CHUNK_SIZE * CHUNK_SIZE + (y - 1) * CHUNK_SIZE + x]
                                .is_solid()
                        },
                        if z == CHUNK_SIZE - 1 {
                            chunk_pz.cubes[y * CHUNK_SIZE + x].is_solid()
                        } else {
                            chunk.cubes[(z + 1) * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x]
                                .is_solid()
                        },
                        if z == 0 {
                            chunk_nz.cubes
                                [(CHUNK_SIZE - 1) * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x]
                                .is_solid()
                        } else {
                            chunk.cubes[(z - 1) * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x]
                                .is_solid()
                        },
                        vertex_data.len(),
                    );
                    vertex_data.append(&mut tmp_vertex_data);
                    index_data.append(&mut tmp_index_data);
                }
            }
        }
    }
    (vertex_data, index_data)
}

pub fn create_vertices(
    x: f32,
    y: f32,
    z: f32,
    px: bool,
    nx: bool,
    py: bool,
    ny: bool,
    pz: bool,
    nz: bool,
    index: usize,
) -> (Vec<Vertex>, Vec<u16>) {
    let offset = index as u16;

    let mut vertex_data = Vec::<Vertex>::new();
    let mut index_data = Vec::<u16>::new();

    if !px {
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 0.0], [0.0, 0.0]));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 0.0], [1.0, 0.0]));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 1.0], [1.0, 1.0]));
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 1.0], [0.0, 1.0]));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }

    if !nx {
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 1.0], [1.0, 0.0]));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 1.0], [0.0, 0.0]));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 0.0], [0.0, 1.0]));
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 0.0], [1.0, 1.0]));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }

    if !py {
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 0.0], [1.0, 0.0]));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 0.0], [0.0, 0.0]));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 1.0], [0.0, 1.0]));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 1.0], [1.0, 1.0]));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }

    if !ny {
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 1.0], [0.0, 0.0]));
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 1.0], [1.0, 0.0]));
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 0.0], [1.0, 1.0]));
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 0.0], [0.0, 1.0]));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }

    if !pz {
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 1.0], [0.0, 0.0]));
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 1.0], [1.0, 0.0]));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 1.0], [1.0, 1.0]));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 1.0], [0.0, 1.0]));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }

    if !nz {
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 0.0], [1.0, 0.0]));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 0.0], [0.0, 0.0]));
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 0.0], [0.0, 1.0]));
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 0.0], [1.0, 1.0]));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }

    (vertex_data, index_data)
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
