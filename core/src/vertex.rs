use bytemuck::{Pod, Zeroable};
use map_types::{
    Chunk, Cube, Custom, FilteredSolid, Harvestable, Plantlike, Solid, Translucent, CHUNK_SIZE,
    MAP_HEIGHT,
};

use crate::terrain_manager::Mesh;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
    _filter_tex_coord: [f32; 2],
    _filter_color: [f32; 4],
}

pub fn vertex(pos: [f32; 3], tc: [f32; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0], pos[1], pos[2], 1.0],
        _tex_coord: tc,
        _filter_tex_coord: [0.0, 0.0],
        _filter_color: [0.0, 0.0, 0.0, 0.0],
    }
}

pub fn filtered_vertex(pos: [f32; 3], tc: [f32; 2], ftc: [f32; 2], fc: [f32; 4]) -> Vertex {
    Vertex {
        _pos: [pos[0], pos[1], pos[2], 1.0],
        _tex_coord: tc,
        _filter_tex_coord: ftc,
        _filter_color: fc,
    }
}

pub fn create_mesh_for_chunk(
    chunk: &Chunk,
    chunk_x: i32,
    chunk_y: i32,
    chunk_px: &Chunk,
    chunk_nx: &Chunk,
    chunk_py: &Chunk,
    chunk_ny: &Chunk,
) -> Mesh {
    let x_offset = chunk_x * CHUNK_SIZE as i32;
    let y_offset = chunk_y * CHUNK_SIZE as i32;

    let mut opaque_buffers = Vec::new();
    let mut translucent_buffers = Vec::new();
    let mut vertex_data_for_opaque = Vec::<Vertex>::new();
    let mut index_data_for_opaque = Vec::<u16>::new();
    let mut vertex_data_for_translucent = Vec::<Vertex>::new();
    let mut index_data_for_translucent = Vec::<u16>::new();
    for z in 0..MAP_HEIGHT {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let actual_x = x_offset + x as i32;
                let actual_y = y_offset + y as i32;
                let actual_z = z as i32;
                match chunk.cubes[z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x] {
                    Cube::Empty => {}
                    Cube::Solid(solid) => {
                        if vertex_data_for_opaque.len() > 60000 {
                            opaque_buffers.push((vertex_data_for_opaque, index_data_for_opaque));
                            vertex_data_for_opaque = Vec::new();
                            index_data_for_opaque = Vec::new();
                        }
                        let (mut tmp_vertex_data, mut tmp_index_data) = create_vertices_for_solid(
                            solid,
                            actual_x as f32,
                            actual_y as f32,
                            actual_z as f32,
                            if x == CHUNK_SIZE - 1 {
                                chunk_px.cubes[z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE]
                                    .is_solid()
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
                                chunk_ny.cubes[z * CHUNK_SIZE * CHUNK_SIZE
                                    + (CHUNK_SIZE - 1) * CHUNK_SIZE
                                    + x]
                                    .is_solid()
                            } else {
                                chunk.cubes[z * CHUNK_SIZE * CHUNK_SIZE + (y - 1) * CHUNK_SIZE + x]
                                    .is_solid()
                            },
                            if z == MAP_HEIGHT - 1 {
                                false
                            } else {
                                chunk.cubes[(z + 1) * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x]
                                    .is_solid()
                            },
                            if z == 0 {
                                false
                            } else {
                                chunk.cubes[(z - 1) * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x]
                                    .is_solid()
                            },
                            vertex_data_for_opaque.len(),
                        );
                        vertex_data_for_opaque.append(&mut tmp_vertex_data);
                        index_data_for_opaque.append(&mut tmp_index_data);
                    }
                    Cube::Translucent(translucent) => {
                        if vertex_data_for_translucent.len() > 60000 {
                            translucent_buffers
                                .push((vertex_data_for_translucent, index_data_for_translucent));
                            vertex_data_for_translucent = Vec::new();
                            index_data_for_translucent = Vec::new();
                        }
                        let (mut tmp_vertex_data, mut tmp_index_data) =
                            create_vertices_for_translucent(
                                translucent,
                                actual_x as f32,
                                actual_y as f32,
                                actual_z as f32,
                                if x == CHUNK_SIZE - 1 {
                                    chunk_px.cubes[z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE]
                                        .is_translucent_or_solid()
                                } else {
                                    chunk.cubes
                                        [z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x + 1]
                                        .is_translucent_or_solid()
                                },
                                if x == 0 {
                                    chunk_nx.cubes[z * CHUNK_SIZE * CHUNK_SIZE
                                        + y * CHUNK_SIZE
                                        + CHUNK_SIZE
                                        - 1]
                                    .is_translucent_or_solid()
                                } else {
                                    chunk.cubes
                                        [z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x - 1]
                                        .is_translucent_or_solid()
                                },
                                if y == CHUNK_SIZE - 1 {
                                    chunk_py.cubes[z * CHUNK_SIZE * CHUNK_SIZE + x]
                                        .is_translucent_or_solid()
                                } else {
                                    chunk.cubes
                                        [z * CHUNK_SIZE * CHUNK_SIZE + (y + 1) * CHUNK_SIZE + x]
                                        .is_translucent_or_solid()
                                },
                                if y == 0 {
                                    chunk_ny.cubes[z * CHUNK_SIZE * CHUNK_SIZE
                                        + (CHUNK_SIZE - 1) * CHUNK_SIZE
                                        + x]
                                        .is_translucent_or_solid()
                                } else {
                                    chunk.cubes
                                        [z * CHUNK_SIZE * CHUNK_SIZE + (y - 1) * CHUNK_SIZE + x]
                                        .is_translucent_or_solid()
                                },
                                if z == MAP_HEIGHT - 1 {
                                    false
                                } else {
                                    chunk.cubes
                                        [(z + 1) * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x]
                                        .is_translucent_or_solid()
                                },
                                if z == 0 {
                                    false
                                } else {
                                    chunk.cubes
                                        [(z - 1) * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x]
                                        .is_translucent_or_solid()
                                },
                                vertex_data_for_translucent.len(),
                            );
                        vertex_data_for_translucent.append(&mut tmp_vertex_data);
                        index_data_for_translucent.append(&mut tmp_index_data);
                    }
                    Cube::FilteredSolid(filtered_solid) => {
                        if vertex_data_for_opaque.len() > 60000 {
                            opaque_buffers.push((vertex_data_for_opaque, index_data_for_opaque));
                            vertex_data_for_opaque = Vec::new();
                            index_data_for_opaque = Vec::new();
                        }
                        let (mut tmp_vertex_data, mut tmp_index_data) =
                            create_vertices_for_filtered_solid(
                                filtered_solid,
                                actual_x as f32,
                                actual_y as f32,
                                actual_z as f32,
                                if x == CHUNK_SIZE - 1 {
                                    chunk_px.cubes[z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE]
                                        .is_solid()
                                } else {
                                    chunk.cubes
                                        [z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x + 1]
                                        .is_solid()
                                },
                                if x == 0 {
                                    chunk_nx.cubes[z * CHUNK_SIZE * CHUNK_SIZE
                                        + y * CHUNK_SIZE
                                        + CHUNK_SIZE
                                        - 1]
                                    .is_solid()
                                } else {
                                    chunk.cubes
                                        [z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x - 1]
                                        .is_solid()
                                },
                                if y == CHUNK_SIZE - 1 {
                                    chunk_py.cubes[z * CHUNK_SIZE * CHUNK_SIZE + x].is_solid()
                                } else {
                                    chunk.cubes
                                        [z * CHUNK_SIZE * CHUNK_SIZE + (y + 1) * CHUNK_SIZE + x]
                                        .is_solid()
                                },
                                if y == 0 {
                                    chunk_ny.cubes[z * CHUNK_SIZE * CHUNK_SIZE
                                        + (CHUNK_SIZE - 1) * CHUNK_SIZE
                                        + x]
                                        .is_solid()
                                } else {
                                    chunk.cubes
                                        [z * CHUNK_SIZE * CHUNK_SIZE + (y - 1) * CHUNK_SIZE + x]
                                        .is_solid()
                                },
                                if z == MAP_HEIGHT - 1 {
                                    false
                                } else {
                                    chunk.cubes
                                        [(z + 1) * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x]
                                        .is_solid()
                                },
                                if z == 0 {
                                    false
                                } else {
                                    chunk.cubes
                                        [(z - 1) * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x]
                                        .is_solid()
                                },
                                [
                                    chunk.biome_colors[y * CHUNK_SIZE + x],
                                    if x == CHUNK_SIZE - 1 {
                                        chunk_px.biome_colors[y * CHUNK_SIZE]
                                    } else {
                                        chunk.biome_colors[y * CHUNK_SIZE + x + 1]
                                    },
                                    if y == CHUNK_SIZE - 1 {
                                        chunk_py.biome_colors[x]
                                    } else {
                                        chunk.biome_colors[(y + 1) * CHUNK_SIZE + x]
                                    },
                                    if y == CHUNK_SIZE - 1 {
                                        if x == CHUNK_SIZE - 1 {
                                            [0.0, 0.0, 0.0, 0.0] // TODO: fix
                                        } else {
                                            chunk_py.biome_colors[x + 1]
                                        }
                                    } else if x == CHUNK_SIZE - 1 {
                                        chunk_px.biome_colors[(y + 1) * CHUNK_SIZE]
                                    } else {
                                        chunk.biome_colors[(y + 1) * CHUNK_SIZE + x + 1]
                                    },
                                ],
                                vertex_data_for_opaque.len(),
                            );
                        vertex_data_for_opaque.append(&mut tmp_vertex_data);
                        index_data_for_opaque.append(&mut tmp_index_data);
                    }
                    Cube::Plantlike(plantlike) => {
                        if vertex_data_for_translucent.len() > 60000 {
                            translucent_buffers
                                .push((vertex_data_for_translucent, index_data_for_translucent));
                            vertex_data_for_translucent = Vec::new();
                            index_data_for_translucent = Vec::new();
                        }
                        let (mut tmp_vertex_data, mut tmp_index_data) =
                            create_vertices_for_plantlike(
                                plantlike,
                                actual_x as f32,
                                actual_y as f32,
                                actual_z as f32,
                                vertex_data_for_translucent.len(),
                            );
                        vertex_data_for_translucent.append(&mut tmp_vertex_data);
                        index_data_for_translucent.append(&mut tmp_index_data);
                    }
                    Cube::Harvestable(harvestable) => {
                        if vertex_data_for_translucent.len() > 60000 {
                            translucent_buffers
                                .push((vertex_data_for_translucent, index_data_for_translucent));
                            vertex_data_for_translucent = Vec::new();
                            index_data_for_translucent = Vec::new();
                        }
                        let (mut tmp_vertex_data, mut tmp_index_data) =
                            create_vertices_for_harvestable(
                                harvestable,
                                actual_x as f32,
                                actual_y as f32,
                                actual_z as f32,
                                vertex_data_for_translucent.len(),
                            );
                        vertex_data_for_translucent.append(&mut tmp_vertex_data);
                        index_data_for_translucent.append(&mut tmp_index_data);
                    }
                    Cube::Custom(custom) => {
                        if vertex_data_for_opaque.len() > 60000 {
                            opaque_buffers.push((vertex_data_for_opaque, index_data_for_opaque));
                            vertex_data_for_opaque = Vec::new();
                            index_data_for_opaque = Vec::new();
                        }
                        if vertex_data_for_translucent.len() > 60000 {
                            translucent_buffers
                                .push((vertex_data_for_translucent, index_data_for_translucent));
                            vertex_data_for_translucent = Vec::new();
                            index_data_for_translucent = Vec::new();
                        }
                        let (
                            (mut tmp_vertex_data_for_opaque, mut tmp_index_data_for_opaque),
                            (
                                mut tmp_vertex_data_for_translucent,
                                mut tmp_index_data_for_translucent,
                            ),
                        ) = create_vertices_for_custom(
                            custom,
                            actual_x as f32,
                            actual_y as f32,
                            actual_z as f32,
                            vertex_data_for_opaque.len(),
                            vertex_data_for_translucent.len(),
                        );
                        vertex_data_for_opaque.append(&mut tmp_vertex_data_for_opaque);
                        index_data_for_opaque.append(&mut tmp_index_data_for_opaque);
                        vertex_data_for_translucent.append(&mut tmp_vertex_data_for_translucent);
                        index_data_for_translucent.append(&mut tmp_index_data_for_translucent);
                    }
                }
            }
        }
    }
    if !index_data_for_opaque.is_empty() {
        opaque_buffers.push((vertex_data_for_opaque, index_data_for_opaque));
    }
    if !index_data_for_translucent.is_empty() {
        translucent_buffers.push((vertex_data_for_translucent, index_data_for_translucent));
    }
    Mesh {
        opaque_buffers,
        translucent_buffers,
    }
}

pub fn create_vertices_for_solid(
    solid: Solid,
    x: f32,
    y: f32,
    z: f32,
    px_is_solid: bool,
    nx_is_solid: bool,
    py_is_solid: bool,
    ny_is_solid: bool,
    pz_is_solid: bool,
    nz_is_solid: bool,
    index: usize,
) -> (Vec<Vertex>, Vec<u16>) {
    let offset = index as u16;

    let mut vertex_data = Vec::<Vertex>::new();
    let mut index_data = Vec::<u16>::new();

    if !px_is_solid {
        let [a, b, c, d] = solid.tex_coord_px();
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 0.0], a));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 0.0], b));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 1.0], c));
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 1.0], d));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    if !nx_is_solid {
        let [a, b, c, d] = solid.tex_coord_nx();
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 1.0], a));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 1.0], b));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 0.0], c));
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 0.0], d));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    if !py_is_solid {
        let [a, b, c, d] = solid.tex_coord_py();
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 0.0], a));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 0.0], b));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 1.0], c));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 1.0], d));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    if !ny_is_solid {
        let [a, b, c, d] = solid.tex_coord_ny();
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 1.0], a));
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 1.0], b));
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 0.0], c));
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 0.0], d));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    if !pz_is_solid {
        let [a, b, c, d] = solid.tex_coord_pz();
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 1.0], a));
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 1.0], b));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 1.0], c));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 1.0], d));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    if !nz_is_solid {
        let [a, b, c, d] = solid.tex_coord_nz();
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 0.0], a));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 0.0], b));
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 0.0], c));
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 0.0], d));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    (vertex_data, index_data)
}

pub fn create_vertices_for_translucent(
    translucent: Translucent,
    x: f32,
    y: f32,
    z: f32,
    px_is_translucent_or_solid: bool,
    nx_is_translucent_or_solid: bool,
    py_is_translucent_or_solid: bool,
    ny_is_translucent_or_solid: bool,
    pz_is_translucent_or_solid: bool,
    nz_is_translucent_or_solid: bool,
    index: usize,
) -> (Vec<Vertex>, Vec<u16>) {
    let offset = index as u16;

    let mut vertex_data = Vec::<Vertex>::new();
    let mut index_data = Vec::<u16>::new();

    if !px_is_translucent_or_solid {
        let [a, b, c, d] = translucent.tex_coord();
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 0.0], a));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 0.0], b));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 1.0], c));
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 1.0], d));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    if !nx_is_translucent_or_solid {
        let [a, b, c, d] = translucent.tex_coord();
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 1.0], a));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 1.0], b));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 0.0], c));
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 0.0], d));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    if !py_is_translucent_or_solid {
        let [a, b, c, d] = translucent.tex_coord();
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 0.0], a));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 0.0], b));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 1.0], c));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 1.0], d));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    if !ny_is_translucent_or_solid {
        let [a, b, c, d] = translucent.tex_coord();
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 1.0], a));
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 1.0], b));
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 0.0], c));
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 0.0], d));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    if !pz_is_translucent_or_solid {
        let [a, b, c, d] = translucent.tex_coord();
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 1.0], a));
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 1.0], b));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 1.0], c));
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 1.0], d));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    if !nz_is_translucent_or_solid {
        let [a, b, c, d] = translucent.tex_coord();
        vertex_data.push(vertex([x + 0.0, y + 1.0, z + 0.0], a));
        vertex_data.push(vertex([x + 1.0, y + 1.0, z + 0.0], b));
        vertex_data.push(vertex([x + 1.0, y + 0.0, z + 0.0], c));
        vertex_data.push(vertex([x + 0.0, y + 0.0, z + 0.0], d));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    (vertex_data, index_data)
}

pub fn create_vertices_for_filtered_solid(
    filtered_solid: FilteredSolid,
    x: f32,
    y: f32,
    z: f32,
    px_is_solid: bool,
    nx_is_solid: bool,
    py_is_solid: bool,
    ny_is_solid: bool,
    pz_is_solid: bool,
    nz_is_solid: bool,
    biome_color: [[f32; 4]; 4],
    index: usize,
) -> (Vec<Vertex>, Vec<u16>) {
    let offset = index as u16;

    let mut vertex_data = Vec::<Vertex>::new();
    let mut index_data = Vec::<u16>::new();

    if !px_is_solid {
        let ([a, b, c, d], [e, f, g, h]) = filtered_solid.extras_px();
        vertex_data.push(filtered_vertex(
            [x + 1.0, y + 0.0, z + 0.0],
            a,
            e,
            biome_color[1],
        ));
        vertex_data.push(filtered_vertex(
            [x + 1.0, y + 1.0, z + 0.0],
            b,
            f,
            biome_color[3],
        ));
        vertex_data.push(filtered_vertex(
            [x + 1.0, y + 1.0, z + 1.0],
            c,
            g,
            biome_color[3],
        ));
        vertex_data.push(filtered_vertex(
            [x + 1.0, y + 0.0, z + 1.0],
            d,
            h,
            biome_color[1],
        ));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    if !nx_is_solid {
        let ([a, b, c, d], [e, f, g, h]) = filtered_solid.extras_nx();
        vertex_data.push(filtered_vertex(
            [x + 0.0, y + 0.0, z + 1.0],
            a,
            e,
            biome_color[0],
        ));
        vertex_data.push(filtered_vertex(
            [x + 0.0, y + 1.0, z + 1.0],
            b,
            f,
            biome_color[2],
        ));
        vertex_data.push(filtered_vertex(
            [x + 0.0, y + 1.0, z + 0.0],
            c,
            g,
            biome_color[2],
        ));
        vertex_data.push(filtered_vertex(
            [x + 0.0, y + 0.0, z + 0.0],
            d,
            h,
            biome_color[0],
        ));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    if !py_is_solid {
        let ([a, b, c, d], [e, f, g, h]) = filtered_solid.extras_py();
        vertex_data.push(filtered_vertex(
            [x + 1.0, y + 1.0, z + 0.0],
            a,
            e,
            biome_color[3],
        ));
        vertex_data.push(filtered_vertex(
            [x + 0.0, y + 1.0, z + 0.0],
            b,
            f,
            biome_color[2],
        ));
        vertex_data.push(filtered_vertex(
            [x + 0.0, y + 1.0, z + 1.0],
            c,
            g,
            biome_color[2],
        ));
        vertex_data.push(filtered_vertex(
            [x + 1.0, y + 1.0, z + 1.0],
            d,
            h,
            biome_color[3],
        ));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    if !ny_is_solid {
        let ([a, b, c, d], [e, f, g, h]) = filtered_solid.extras_ny();
        vertex_data.push(filtered_vertex(
            [x + 1.0, y + 0.0, z + 1.0],
            a,
            e,
            biome_color[1],
        ));
        vertex_data.push(filtered_vertex(
            [x + 0.0, y + 0.0, z + 1.0],
            b,
            f,
            biome_color[0],
        ));
        vertex_data.push(filtered_vertex(
            [x + 0.0, y + 0.0, z + 0.0],
            c,
            g,
            biome_color[0],
        ));
        vertex_data.push(filtered_vertex(
            [x + 1.0, y + 0.0, z + 0.0],
            d,
            h,
            biome_color[1],
        ));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    if !pz_is_solid {
        let ([a, b, c, d], [e, f, g, h]) = filtered_solid.extras_pz();
        vertex_data.push(filtered_vertex(
            [x + 0.0, y + 0.0, z + 1.0],
            a,
            e,
            biome_color[0],
        ));
        vertex_data.push(filtered_vertex(
            [x + 1.0, y + 0.0, z + 1.0],
            b,
            f,
            biome_color[1],
        ));
        vertex_data.push(filtered_vertex(
            [x + 1.0, y + 1.0, z + 1.0],
            c,
            g,
            biome_color[3],
        ));
        vertex_data.push(filtered_vertex(
            [x + 0.0, y + 1.0, z + 1.0],
            d,
            h,
            biome_color[2],
        ));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    if !nz_is_solid {
        let ([a, b, c, d], [e, f, g, h]) = filtered_solid.extras_nz();
        vertex_data.push(filtered_vertex(
            [x + 0.0, y + 1.0, z + 0.0],
            a,
            e,
            biome_color[2],
        ));
        vertex_data.push(filtered_vertex(
            [x + 1.0, y + 1.0, z + 0.0],
            b,
            f,
            biome_color[3],
        ));
        vertex_data.push(filtered_vertex(
            [x + 1.0, y + 0.0, z + 0.0],
            c,
            g,
            biome_color[1],
        ));
        vertex_data.push(filtered_vertex(
            [x + 0.0, y + 0.0, z + 0.0],
            d,
            h,
            biome_color[0],
        ));
        index_data.push(offset + vertex_data.len() as u16 - 4);
        index_data.push(offset + vertex_data.len() as u16 - 3);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 2);
        index_data.push(offset + vertex_data.len() as u16 - 1);
        index_data.push(offset + vertex_data.len() as u16 - 4);
    }
    (vertex_data, index_data)
}

pub fn create_vertices_for_plantlike(
    plantlike: Plantlike,
    x: f32,
    y: f32,
    z: f32,
    index: usize,
) -> (Vec<Vertex>, Vec<u16>) {
    let offset = index as u16;

    let mut vertex_data = Vec::<Vertex>::new();
    let mut index_data = Vec::<u16>::new();

    let [a, b, c, d] = plantlike.tex_coord();
    vertex_data.push(vertex([x + 1.0, y + 0.0, z + 0.0], a));
    vertex_data.push(vertex([x + 0.0, y + 1.0, z + 0.0], b));
    vertex_data.push(vertex([x + 0.0, y + 1.0, z + 1.0], c));
    vertex_data.push(vertex([x + 1.0, y + 0.0, z + 1.0], d));
    vertex_data.push(vertex([x + 0.0, y + 0.0, z + 0.0], a));
    vertex_data.push(vertex([x + 1.0, y + 1.0, z + 0.0], b));
    vertex_data.push(vertex([x + 1.0, y + 1.0, z + 1.0], c));
    vertex_data.push(vertex([x + 0.0, y + 0.0, z + 1.0], d));
    index_data.push(offset + vertex_data.len() as u16 - 8);
    index_data.push(offset + vertex_data.len() as u16 - 7);
    index_data.push(offset + vertex_data.len() as u16 - 6);
    index_data.push(offset + vertex_data.len() as u16 - 6);
    index_data.push(offset + vertex_data.len() as u16 - 5);
    index_data.push(offset + vertex_data.len() as u16 - 8);
    index_data.push(offset + vertex_data.len() as u16 - 4);
    index_data.push(offset + vertex_data.len() as u16 - 3);
    index_data.push(offset + vertex_data.len() as u16 - 2);
    index_data.push(offset + vertex_data.len() as u16 - 2);
    index_data.push(offset + vertex_data.len() as u16 - 1);
    index_data.push(offset + vertex_data.len() as u16 - 4);
    (vertex_data, index_data)
}

pub fn create_vertices_for_harvestable(
    harvestable: Harvestable,
    x: f32,
    y: f32,
    z: f32,
    index: usize,
) -> (Vec<Vertex>, Vec<u16>) {
    let offset = index as u16;

    let mut vertex_data = Vec::<Vertex>::new();
    let mut index_data = Vec::<u16>::new();

    let [a, b, c, d] = harvestable.tex_coord();
    vertex_data.push(vertex([x + 0.75, y + 0.0, z + 0.0], a));
    vertex_data.push(vertex([x + 0.75, y + 1.0, z + 0.0], b));
    vertex_data.push(vertex([x + 0.75, y + 1.0, z + 1.0], c));
    vertex_data.push(vertex([x + 0.75, y + 0.0, z + 1.0], d));
    index_data.push(offset + vertex_data.len() as u16 - 4);
    index_data.push(offset + vertex_data.len() as u16 - 3);
    index_data.push(offset + vertex_data.len() as u16 - 2);
    index_data.push(offset + vertex_data.len() as u16 - 2);
    index_data.push(offset + vertex_data.len() as u16 - 1);
    index_data.push(offset + vertex_data.len() as u16 - 4);
    vertex_data.push(vertex([x + 0.25, y + 1.0, z + 0.0], a));
    vertex_data.push(vertex([x + 0.25, y + 0.0, z + 0.0], b));
    vertex_data.push(vertex([x + 0.25, y + 0.0, z + 1.0], c));
    vertex_data.push(vertex([x + 0.25, y + 1.0, z + 1.0], d));
    index_data.push(offset + vertex_data.len() as u16 - 4);
    index_data.push(offset + vertex_data.len() as u16 - 3);
    index_data.push(offset + vertex_data.len() as u16 - 2);
    index_data.push(offset + vertex_data.len() as u16 - 2);
    index_data.push(offset + vertex_data.len() as u16 - 1);
    index_data.push(offset + vertex_data.len() as u16 - 4);
    vertex_data.push(vertex([x + 1.0, y + 0.75, z + 0.0], a));
    vertex_data.push(vertex([x + 0.0, y + 0.75, z + 0.0], b));
    vertex_data.push(vertex([x + 0.0, y + 0.75, z + 1.0], c));
    vertex_data.push(vertex([x + 1.0, y + 0.75, z + 1.0], d));
    index_data.push(offset + vertex_data.len() as u16 - 4);
    index_data.push(offset + vertex_data.len() as u16 - 3);
    index_data.push(offset + vertex_data.len() as u16 - 2);
    index_data.push(offset + vertex_data.len() as u16 - 2);
    index_data.push(offset + vertex_data.len() as u16 - 1);
    index_data.push(offset + vertex_data.len() as u16 - 4);
    vertex_data.push(vertex([x + 0.0, y + 0.25, z + 0.0], a));
    vertex_data.push(vertex([x + 1.0, y + 0.25, z + 0.0], b));
    vertex_data.push(vertex([x + 1.0, y + 0.25, z + 1.0], c));
    vertex_data.push(vertex([x + 0.0, y + 0.25, z + 1.0], d));
    index_data.push(offset + vertex_data.len() as u16 - 4);
    index_data.push(offset + vertex_data.len() as u16 - 3);
    index_data.push(offset + vertex_data.len() as u16 - 2);
    index_data.push(offset + vertex_data.len() as u16 - 2);
    index_data.push(offset + vertex_data.len() as u16 - 1);
    index_data.push(offset + vertex_data.len() as u16 - 4);
    (vertex_data, index_data)
}

pub fn create_vertices_for_custom(
    custom: Custom,
    x: f32,
    y: f32,
    z: f32,
    opaque_index: usize,
    translucent_index: usize,
) -> ((Vec<Vertex>, Vec<u16>), (Vec<Vertex>, Vec<u16>)) {
    // let opaque_offset = opaque_index as u16;
    let translucent_offset = translucent_index as u16;

    let /* mut */ vertex_data_opaque = Vec::<Vertex>::new();
    let /* mut */ index_data_opaque = Vec::<u16>::new();
    let mut vertex_data_translucent = Vec::<Vertex>::new();
    let mut index_data_translucent = Vec::<u16>::new();

    match custom {
        Custom::Cactus => {
            vertex_data_translucent.push(vertex([x + 0.9375, y + 0.0, z + 0.0], [7.0, 5.0]));
            vertex_data_translucent.push(vertex([x + 0.9375, y + 1.0, z + 0.0], [6.0, 5.0]));
            vertex_data_translucent.push(vertex([x + 0.9375, y + 1.0, z + 1.0], [6.0, 4.0]));
            vertex_data_translucent.push(vertex([x + 0.9375, y + 0.0, z + 1.0], [7.0, 4.0]));
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 4);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 3);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 2);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 2);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 1);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 4);
            vertex_data_translucent.push(vertex([x + 0.0625, y + 0.0, z + 1.0], [7.0, 4.0]));
            vertex_data_translucent.push(vertex([x + 0.0625, y + 1.0, z + 1.0], [6.0, 4.0]));
            vertex_data_translucent.push(vertex([x + 0.0625, y + 1.0, z + 0.0], [6.0, 5.0]));
            vertex_data_translucent.push(vertex([x + 0.0625, y + 0.0, z + 0.0], [7.0, 5.0]));
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 4);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 3);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 2);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 2);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 1);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 4);
            vertex_data_translucent.push(vertex([x + 1.0, y + 0.9375, z + 0.0], [6.0, 5.0]));
            vertex_data_translucent.push(vertex([x + 0.0, y + 0.9375, z + 0.0], [7.0, 5.0]));
            vertex_data_translucent.push(vertex([x + 0.0, y + 0.9375, z + 1.0], [7.0, 4.0]));
            vertex_data_translucent.push(vertex([x + 1.0, y + 0.9375, z + 1.0], [6.0, 4.0]));
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 4);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 3);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 2);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 2);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 1);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 4);
            vertex_data_translucent.push(vertex([x + 1.0, y + 0.0625, z + 1.0], [6.0, 4.0]));
            vertex_data_translucent.push(vertex([x + 0.0, y + 0.0625, z + 1.0], [7.0, 4.0]));
            vertex_data_translucent.push(vertex([x + 0.0, y + 0.0625, z + 0.0], [7.0, 5.0]));
            vertex_data_translucent.push(vertex([x + 1.0, y + 0.0625, z + 0.0], [6.0, 5.0]));
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 4);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 3);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 2);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 2);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 1);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 4);
            vertex_data_translucent.push(vertex([x + 0.0, y + 0.0, z + 1.0], [6.0, 5.0]));
            vertex_data_translucent.push(vertex([x + 1.0, y + 0.0, z + 1.0], [5.0, 5.0]));
            vertex_data_translucent.push(vertex([x + 1.0, y + 1.0, z + 1.0], [5.0, 4.0]));
            vertex_data_translucent.push(vertex([x + 0.0, y + 1.0, z + 1.0], [6.0, 4.0]));
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 4);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 3);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 2);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 2);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 1);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 4);
            vertex_data_translucent.push(vertex([x + 0.0, y + 1.0, z + 0.0], [8.0, 5.0]));
            vertex_data_translucent.push(vertex([x + 1.0, y + 1.0, z + 0.0], [7.0, 5.0]));
            vertex_data_translucent.push(vertex([x + 1.0, y + 0.0, z + 0.0], [7.0, 4.0]));
            vertex_data_translucent.push(vertex([x + 0.0, y + 0.0, z + 0.0], [8.0, 4.0]));
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 4);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 3);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 2);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 2);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 1);
            index_data_translucent
                .push(translucent_offset + vertex_data_translucent.len() as u16 - 4);
        }
    }

    (
        (vertex_data_opaque, index_data_opaque),
        (vertex_data_translucent, index_data_translucent),
    )
}
