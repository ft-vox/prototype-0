use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use ft_vox_prototype_0_map_types::{Chunk, CHUNK_SIZE};

use crate::vertex::Vertex;
use crate::{get_coords, TerrainWorker};

pub struct ChunkCache<T: TerrainWorker> {
    cache: Arc<Mutex<Cache>>,
    eye: (f32, f32, f32),
    terrain_worker: T,
}

struct Cache {
    pub chunk_loading: HashSet<(i32, i32, i32)>,
    pub chunk_cache: Vec<Option<Arc<Chunk>>>,

    pub mesh_loading: HashSet<(i32, i32, i32)>,
    pub mesh_cache: Vec<Option<Arc<(Vec<Vertex>, Vec<u16>)>>>,

    pub cache_distance: usize,
    pub coords: Vec<(i32, i32, i32)>,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub eye_x_upper: bool,
    pub eye_y_upper: bool,
    pub eye_z_upper: bool,
}

impl Cache {
    pub fn get_chunk(&self, x: i32, y: i32, z: i32) -> Option<Arc<Chunk>> {
        let size = self.cache_distance * 2 + 2;

        let min_x = self.x - self.cache_distance as i32 - if self.eye_x_upper { 0 } else { 1 };
        let max_x = self.x + self.cache_distance as i32 + if self.eye_x_upper { 1 } else { 0 };
        if min_x > x || x > max_x {
            return None;
        }
        let x = x.rem_euclid(size as i32) as usize;

        let min_y = self.y - self.cache_distance as i32 - if self.eye_y_upper { 0 } else { 1 };
        let max_y = self.y + self.cache_distance as i32 + if self.eye_y_upper { 1 } else { 0 };
        if min_y > y || y > max_y {
            return None;
        }
        let y = y.rem_euclid(size as i32) as usize;

        let min_z = self.z - self.cache_distance as i32 - if self.eye_z_upper { 0 } else { 1 };
        let max_z = self.z + self.cache_distance as i32 + if self.eye_z_upper { 1 } else { 0 };
        if min_z > z || z > max_z {
            return None;
        }
        let z = z.rem_euclid(size as i32) as usize;

        self.chunk_cache[z * size * size + y * size + x].clone()
    }

    pub fn set_chunk(&mut self, x: i32, y: i32, z: i32, chunk: Option<Arc<Chunk>>) {
        let size = self.cache_distance * 2 + 2;

        let min_x = self.x - self.cache_distance as i32 - if self.eye_x_upper { 0 } else { 1 };
        let max_x = self.x + self.cache_distance as i32 + if self.eye_x_upper { 1 } else { 0 };
        if min_x > x || x > max_x {
            return;
        }
        let x = x.rem_euclid(size as i32) as usize;

        let min_y = self.y - self.cache_distance as i32 - if self.eye_y_upper { 0 } else { 1 };
        let max_y = self.y + self.cache_distance as i32 + if self.eye_y_upper { 1 } else { 0 };
        if min_y > y || y > max_y {
            return;
        }
        let y = y.rem_euclid(size as i32) as usize;

        let min_z = self.z - self.cache_distance as i32 - if self.eye_z_upper { 0 } else { 1 };
        let max_z = self.z + self.cache_distance as i32 + if self.eye_z_upper { 1 } else { 0 };
        if min_z > z || z > max_z {
            return;
        }
        let z = z.rem_euclid(size as i32) as usize;

        self.chunk_cache[z * size * size + y * size + x] = chunk;
    }

    pub fn get_mesh(&self, x: i32, y: i32, z: i32) -> Option<Arc<(Vec<Vertex>, Vec<u16>)>> {
        let size = self.cache_distance * 2 + 2;

        let min_x = self.x - self.cache_distance as i32 - if self.eye_x_upper { 0 } else { 1 };
        let max_x = self.x + self.cache_distance as i32 + if self.eye_x_upper { 1 } else { 0 };
        if min_x > x || x > max_x {
            return None;
        }
        let x = x.rem_euclid(size as i32) as usize;

        let min_y = self.y - self.cache_distance as i32 - if self.eye_y_upper { 0 } else { 1 };
        let max_y = self.y + self.cache_distance as i32 + if self.eye_y_upper { 1 } else { 0 };
        if min_y > y || y > max_y {
            return None;
        }
        let y = y.rem_euclid(size as i32) as usize;

        let min_z = self.z - self.cache_distance as i32 - if self.eye_z_upper { 0 } else { 1 };
        let max_z = self.z + self.cache_distance as i32 + if self.eye_z_upper { 1 } else { 0 };
        if min_z > z || z > max_z {
            return None;
        }
        let z = z.rem_euclid(size as i32) as usize;

        self.mesh_cache[z * size * size + y * size + x].clone()
    }

    pub fn set_mesh(&mut self, x: i32, y: i32, z: i32, mesh: Option<Arc<(Vec<Vertex>, Vec<u16>)>>) {
        let size = self.cache_distance * 2 + 2;

        let min_x = self.x - self.cache_distance as i32 - if self.eye_x_upper { 0 } else { 1 };
        let max_x = self.x + self.cache_distance as i32 + if self.eye_x_upper { 1 } else { 0 };
        if min_x > x || x > max_x {
            return;
        }
        let x = x.rem_euclid(size as i32) as usize;

        let min_y = self.y - self.cache_distance as i32 - if self.eye_y_upper { 0 } else { 1 };
        let max_y = self.y + self.cache_distance as i32 + if self.eye_y_upper { 1 } else { 0 };
        if min_y > y || y > max_y {
            return;
        }
        let y = y.rem_euclid(size as i32) as usize;

        let min_z = self.z - self.cache_distance as i32 - if self.eye_z_upper { 0 } else { 1 };
        let max_z = self.z + self.cache_distance as i32 + if self.eye_z_upper { 1 } else { 0 };
        if min_z > z || z > max_z {
            return;
        }
        let z = z.rem_euclid(size as i32) as usize;

        self.mesh_cache[z * size * size + y * size + x] = mesh;
    }

    fn reset(&mut self) {
        let size = self.cache_distance * 2 + 2;
        self.chunk_cache = vec![None; size * size * size];
        self.mesh_cache = vec![None; size * size * size];
        self.chunk_loading.clear();
        self.mesh_loading.clear();
    }

    fn get_available(&self) -> Vec<((i32, i32, i32), Arc<(Vec<Vertex>, Vec<u16>)>)> {
        self.coords
            .iter()
            .map(|&(x, y, z)| (x + self.x, y + self.y, z + self.z))
            .filter_map(|(x, y, z)| self.get_mesh(x, y, z).map(|mesh| ((x, y, z), mesh)))
            .collect()
    }
}

impl<T: TerrainWorker> ChunkCache<T> {
    pub fn new(cache_distance: usize, eye: (f32, f32, f32)) -> Self {
        let size = cache_distance * 2 + 2;
        let (x, y, z) = eye;

        let mut result = Self {
            cache: Arc::new(Mutex::new(Cache {
                chunk_loading: HashSet::new(),
                chunk_cache: vec![None; size * size * size],
                mesh_loading: HashSet::new(),
                mesh_cache: vec![None; size * size * size],
                cache_distance,
                coords: Self::calculate_coords(cache_distance as f32),
                x: (x / CHUNK_SIZE as f32).floor() as i32,
                y: (y / CHUNK_SIZE as f32).floor() as i32,
                z: (z / CHUNK_SIZE as f32).floor() as i32,
                eye_x_upper: x % CHUNK_SIZE as f32 > CHUNK_SIZE as f32 / 2.0,
                eye_y_upper: y % CHUNK_SIZE as f32 > CHUNK_SIZE as f32 / 2.0,
                eye_z_upper: z % CHUNK_SIZE as f32 > CHUNK_SIZE as f32 / 2.0,
            })),
            eye,
            terrain_worker: T::new(
                Arc::new(Mutex::new(|| None)),
                Arc::new(Mutex::new(|_pos, _chunk| ())),
                Arc::new(Mutex::new(|| None)),
                Arc::new(Mutex::new(|_pos, _mesh| ())),
            ),
        };
        result.init();

        result
    }

    fn init(&mut self) {
        self.terrain_worker = T::new(
            Arc::new(Mutex::new({
                let cache = self.cache.clone();
                move || {
                    let mut cache = cache.lock().unwrap();
                    let result = cache
                        .coords
                        .iter()
                        .map(|&(x, y, z)| (x + cache.x, y + cache.y, z + cache.z))
                        .find(|&(x, y, z)| {
                            cache.get_chunk(x, y, z).is_none()
                                && !cache.chunk_loading.contains(&(x, y, z))
                        });
                    if let Some((x, y, z)) = result {
                        cache.chunk_loading.insert((x, y, z));
                    }
                    result
                }
            })),
            Arc::new(Mutex::new({
                let cache = self.cache.clone();
                move |(x, y, z), chunk| {
                    let mut cache = cache.lock().unwrap();
                    cache.chunk_loading.remove(&(x, y, z));
                    cache.set_chunk(x, y, z, Some(chunk));
                    cache.mesh_loading.insert((x, y, z));
                }
            })),
            Arc::new(Mutex::new({
                let cache = self.cache.clone();
                move || {
                    let mut cache = cache.lock().unwrap();
                    let mut result: Option<((i32, i32, i32), Vec<Arc<Chunk>>)> = None;

                    for &(x, y, z) in &cache.mesh_loading {
                        let directions = [
                            (0, 0, 0),
                            (1, 0, 0),  // x+1
                            (-1, 0, 0), // x-1
                            (0, 1, 0),  // y+1
                            (0, -1, 0), // y-1
                            (0, 0, 1),  // z+1
                            (0, 0, -1), // z-1
                        ];
                        let mut chunks: Vec<Arc<Chunk>> = Vec::new();
                        for (dx, dy, dz) in directions.iter() {
                            if let Some(chunk) = cache.get_chunk(x + dx, y + dy, z + dz) {
                                chunks.push(chunk.clone());
                            }
                        }
                        if chunks.len() == 7 {
                            result = Some(((x, y, z), chunks));
                            cache.mesh_loading.remove(&(x, y, z));
                            break;
                        }
                    }
                    result
                }
            })),
            Arc::new(Mutex::new({
                let cache = self.cache.clone();
                move |(x, y, z), mesh| {
                    let mut cache = cache.lock().unwrap();
                    cache.set_mesh(x, y, z, Some(mesh));
                }
            })),
        );
    }

    pub fn set_cache_distance(&mut self, new_cache_distance: usize) {
        let mut cache = self.cache.lock().unwrap();

        if cache.cache_distance == new_cache_distance {
            return;
        }

        // TODO: add resize without reset

        cache.cache_distance = new_cache_distance;
        cache.coords = Self::calculate_coords(cache.cache_distance as f32);
        cache.reset();
    }

    pub fn get_available(&self) -> Vec<((i32, i32, i32), Arc<(Vec<Vertex>, Vec<u16>)>)> {
        self.cache.lock().unwrap().get_available()
    }

    pub fn set_eye(&mut self, eye: (f32, f32, f32)) {
        let mut cache = self.cache.lock().unwrap();
        fn upper(value: f32, old: bool) -> bool {
            let value = (value.fract() + 1.0).fract();
            if old {
                value > 0.25
            } else {
                value > 0.75
            }
        }
        let size = cache.cache_distance * 2 + 2;
        let old_eye_chunk_x = cache.x;
        let old_eye_chunk_y = cache.y;
        let old_eye_chunk_z = cache.z;
        let old_eye_x_upper = cache.eye_x_upper;
        let old_eye_y_upper = cache.eye_y_upper;
        let old_eye_z_upper = cache.eye_z_upper;
        let old_min_x =
            old_eye_chunk_x - cache.cache_distance as i32 - if old_eye_x_upper { 0 } else { 1 };
        let old_min_y =
            old_eye_chunk_y - cache.cache_distance as i32 - if old_eye_y_upper { 0 } else { 1 };
        let old_min_z =
            old_eye_chunk_z - cache.cache_distance as i32 - if old_eye_z_upper { 0 } else { 1 };
        let (new_eye_x, new_eye_y, new_eye_z) = eye;
        let new_eye_chunk_x = (new_eye_x / CHUNK_SIZE as f32).floor() as i32;
        let new_eye_chunk_y = (new_eye_y / CHUNK_SIZE as f32).floor() as i32;
        let new_eye_chunk_z = (new_eye_z / CHUNK_SIZE as f32).floor() as i32;
        let new_eye_x_upper = upper(new_eye_x / CHUNK_SIZE as f32, old_eye_x_upper);
        let new_eye_y_upper = upper(new_eye_y / CHUNK_SIZE as f32, old_eye_y_upper);
        let new_eye_z_upper = upper(new_eye_z / CHUNK_SIZE as f32, old_eye_z_upper);
        let new_min_x =
            new_eye_chunk_x - cache.cache_distance as i32 - if new_eye_x_upper { 0 } else { 1 };
        let new_min_y =
            new_eye_chunk_y - cache.cache_distance as i32 - if new_eye_y_upper { 0 } else { 1 };
        let new_min_z =
            new_eye_chunk_z - cache.cache_distance as i32 - if new_eye_z_upper { 0 } else { 1 };
        let new_max_x = new_min_x + size as i32 - 1;
        let new_max_y = new_min_y + size as i32 - 1;
        let new_max_z = new_min_z + size as i32 - 1;

        self.eye = eye;
        cache.eye_x_upper = new_eye_x_upper;
        cache.eye_y_upper = new_eye_y_upper;
        cache.eye_z_upper = new_eye_z_upper;
        cache.x = new_eye_chunk_x;
        cache.y = new_eye_chunk_y;
        cache.z = new_eye_chunk_z;

        match new_min_x - old_min_x {
            0 => {}
            1 => {
                for z in 0..size {
                    for y in 0..size {
                        let x = new_max_x.rem_euclid(size as i32) as usize;
                        cache.chunk_cache[z * size * size + y * size + x] = None;
                        cache.mesh_cache[z * size * size + y * size + x] = None;
                    }
                }
            }
            -1 => {
                for z in 0..size {
                    for y in 0..size {
                        let x = new_min_x.rem_euclid(size as i32) as usize;
                        cache.chunk_cache[z * size * size + y * size + x] = None;
                        cache.mesh_cache[z * size * size + y * size + x] = None;
                    }
                }
            }
            _ => {
                cache.reset();
                return;
            }
        }

        match new_min_y - old_min_y {
            0 => {}
            1 => {
                for z in 0..size {
                    for x in 0..size {
                        let y = new_max_y.rem_euclid(size as i32) as usize;
                        cache.chunk_cache[z * size * size + y * size + x] = None;
                        cache.mesh_cache[z * size * size + y * size + x] = None;
                    }
                }
            }
            -1 => {
                for z in 0..size {
                    for x in 0..size {
                        let y = new_min_y.rem_euclid(size as i32) as usize;
                        cache.chunk_cache[z * size * size + y * size + x] = None;
                        cache.mesh_cache[z * size * size + y * size + x] = None;
                    }
                }
            }
            _ => {
                cache.reset();
                return;
            }
        }

        match new_min_z - old_min_z {
            0 => {}
            1 => {
                for x in 0..size {
                    for y in 0..size {
                        let z = new_max_z.rem_euclid(size as i32) as usize;
                        cache.chunk_cache[z * size * size + y * size + x] = None;
                        cache.mesh_cache[z * size * size + y * size + x] = None;
                    }
                }
            }
            -1 => {
                for x in 0..size {
                    for y in 0..size {
                        let z = new_min_z.rem_euclid(size as i32) as usize;
                        cache.chunk_cache[z * size * size + y * size + x] = None;
                        cache.mesh_cache[z * size * size + y * size + x] = None;
                    }
                }
            }
            _ => {
                cache.reset();
                // return;
            }
        }
    }

    fn calculate_coords(distance: f32) -> Vec<(i32, i32, i32)> {
        let mut result = get_coords(distance);

        fn dst((x, y, z): (i32, i32, i32)) -> i32 {
            x * x + y * y + z * z
        }
        result.sort_unstable_by(|&a, &b| dst(a).cmp(&dst(b)));

        result
    }
}
