use std::{
    collections::{HashSet, VecDeque},
    sync::{Arc, Mutex},
};

use map_types::{Chunk, CHUNK_SIZE};

use crate::{
    get_coords,
    graphics::{DrawCallArgs, MeshBuffer},
    terrain_worker::TerrainWorker,
    Server,
};
use crate::{terrain_worker::TerrainWorkerJob, vertex::Vertex};

pub struct TerrainManager {
    map_cache: Arc<Mutex<MapCache>>,
    mesh_cache: Arc<Mutex<MeshCache>>,
    buffer_cache: BufferCache,
    eye: (f32, f32),
    terrain_worker: TerrainWorker,
    server: Arc<Mutex<Server>>,
}

pub struct Mesh {
    pub opaque_buffers: Vec<(Vec<Vertex>, Vec<u16>)>,
    pub translucent_buffers: Vec<(Vec<Vertex>, Vec<u16>)>,
}

struct MapCache {
    pub chunk_loading: HashSet<(i32, i32)>,
    pub chunks: Vec<Option<Arc<Chunk>>>,

    pub cache_distance: usize,
    pub coords: Vec<(i32, i32)>,
    pub x: i32,
    pub y: i32,
    pub eye_x_upper: bool,
    pub eye_y_upper: bool,
}

impl MapCache {
    pub fn new(cache_distance: usize, eye: (f32, f32)) -> Self {
        let size = cache_distance * 2 + 2;
        let (x, y) = eye;

        MapCache {
            chunk_loading: HashSet::new(),
            chunks: vec![None; size * size],
            cache_distance,
            coords: calculate_coords(cache_distance as f32),
            x: (x / CHUNK_SIZE as f32).floor() as i32,
            y: (y / CHUNK_SIZE as f32).floor() as i32,
            eye_x_upper: x % CHUNK_SIZE as f32 > CHUNK_SIZE as f32 / 2.0,
            eye_y_upper: y % CHUNK_SIZE as f32 > CHUNK_SIZE as f32 / 2.0,
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Arc<Chunk>> {
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

        self.chunks[y * size + x].clone()
    }

    pub fn set(&mut self, x: i32, y: i32, chunk: Option<Arc<Chunk>>) {
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

        self.chunks[y * size + x] = chunk;
    }

    fn reset(&mut self) {
        let size = self.cache_distance * 2 + 2;
        self.chunks = vec![None; size * size];
        self.chunk_loading.clear();
    }
}

struct MeshCache {
    pub mesh_load_request: VecDeque<((i32, i32), Vec<Arc<Chunk>>)>,
    pub meshes: VecDeque<Arc<((i32, i32), Mesh)>>,
}

impl MeshCache {
    pub fn new() -> Self {
        MeshCache {
            mesh_load_request: VecDeque::new(),
            meshes: VecDeque::new(),
        }
    }
}

struct BufferCache {
    pub buffers: Vec<Option<(Arc<Vec<DrawCallArgs>>, Arc<Vec<DrawCallArgs>>)>>,

    pub cache_distance: usize,
    pub coords: Vec<(i32, i32)>,
    pub x: i32,
    pub y: i32,
    pub eye_x_upper: bool,
    pub eye_y_upper: bool,

    pub farthest_distance_sq: i32,
}

impl BufferCache {
    pub fn new(cache_distance: usize, eye: (f32, f32)) -> Self {
        let size = cache_distance * 2 + 2;
        let (x, y) = eye;

        BufferCache {
            buffers: vec![None; size * size],
            cache_distance,
            coords: calculate_coords(cache_distance as f32),
            x: (x / CHUNK_SIZE as f32).floor() as i32,
            y: (y / CHUNK_SIZE as f32).floor() as i32,
            eye_x_upper: x % CHUNK_SIZE as f32 > CHUNK_SIZE as f32 / 2.0,
            eye_y_upper: y % CHUNK_SIZE as f32 > CHUNK_SIZE as f32 / 2.0,
            farthest_distance_sq: 0,
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<(Arc<Vec<DrawCallArgs>>, Arc<Vec<DrawCallArgs>>)> {
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

        self.buffers[y * size + x].clone()
    }

    pub fn set(
        &mut self,
        x: i32,
        y: i32,
        buffer: Option<(Arc<Vec<DrawCallArgs>>, Arc<Vec<DrawCallArgs>>)>,
    ) {
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

        self.buffers[y * size + x] = buffer;
    }

    fn reset(&mut self) {
        let size = self.cache_distance * 2 + 2;
        self.buffers = vec![None; size * size];
    }

    fn get_available(
        &mut self,
        mesh_cache: Arc<Mutex<MeshCache>>,
        process: &mut dyn FnMut(&Mesh) -> (Arc<Vec<DrawCallArgs>>, Arc<Vec<DrawCallArgs>>),
    ) -> Vec<MeshBuffer> {
        fn dst((x, y): (i32, i32)) -> i32 {
            x * x + y * y
        }
        while let Some(item) = mesh_cache.lock().unwrap().meshes.pop_front() {
            let ((x, y), mesh) = &*item;
            self.set(*x, *y, Some(process(mesh)));
            self.farthest_distance_sq =
                std::cmp::max(self.farthest_distance_sq, dst((x - self.x, y - self.y)));
        }
        self.coords
            .iter()
            .map(|&(x, y)| (x + self.x, y + self.y))
            .filter_map(|(x, y)| {
                self.get(x, y).map(|mesh| MeshBuffer {
                    x,
                    y,
                    opaque: mesh.0,
                    translucent: mesh.1,
                })
            })
            .collect()
    }
}

impl TerrainManager {
    pub fn new(cache_distance: usize, eye: (f32, f32), server: Arc<Mutex<Server>>) -> Self {
        let mut result = Self {
            map_cache: Arc::new(Mutex::new(MapCache::new(cache_distance, eye))),
            mesh_cache: Arc::new(Mutex::new(MeshCache::new())),
            buffer_cache: BufferCache::new(cache_distance, eye),
            eye,
            terrain_worker: TerrainWorker::new(
                Arc::new(Mutex::new(|| None)),
                Arc::new(Mutex::new(|_pos, _chunk| ())),
                Arc::new(Mutex::new(|_pos, _mesh| ())),
            ),
            server,
        };
        result.init();

        result
    }

    fn init(&mut self) {
        self.terrain_worker = TerrainWorker::new(
            Arc::new(Mutex::new({
                let map_cache = self.map_cache.clone();
                let mesh_cache = self.mesh_cache.clone();
                move || {
                    let mut map_cache = map_cache.lock().unwrap();
                    if let Some((position, vec)) =
                        mesh_cache.lock().unwrap().mesh_load_request.pop_front()
                    {
                        return Some(TerrainWorkerJob::Mesh {
                            position,
                            zero: vec[0].clone(),
                            positive_x: vec[1].clone(),
                            negative_x: vec[2].clone(),
                            positive_y: vec[3].clone(),
                            negative_y: vec[4].clone(),
                        });
                    }
                    let result = map_cache
                        .coords
                        .iter()
                        .map(|&(x, y)| (x + map_cache.x, y + map_cache.y))
                        .find(|&(x, y)| {
                            map_cache.get(x, y).is_none()
                                && !map_cache.chunk_loading.contains(&(x, y))
                        });
                    if let Some(pos) = result {
                        map_cache.chunk_loading.insert(pos);
                        return Some(TerrainWorkerJob::Map(pos));
                    }
                    None
                }
            })),
            Arc::new(Mutex::new({
                let map_cache = self.map_cache.clone();
                let mesh_cache = self.mesh_cache.clone();
                move |(x, y), chunk| {
                    let mut map_cache = map_cache.lock().unwrap();

                    map_cache.chunk_loading.remove(&(x, y));
                    map_cache.set(x, y, Some(chunk));
                    let directions2 = [
                        (0, 0),  // itself
                        (1, 0),  // x+1
                        (-1, 0), // x-1
                        (0, 1),  // y+1
                        (0, -1), // y-1
                    ];
                    let directions = [
                        (1, 0),  // x+1
                        (-1, 0), // x-1
                        (0, 1),  // y+1
                        (0, -1), // y-1
                    ];
                    for (dx, dy) in directions2.iter() {
                        if let Some(chunk) = map_cache.get(x + dx, y + dy) {
                            let mut chunks5: Vec<Arc<Chunk>> = Vec::new();

                            chunks5.push(chunk.clone());

                            for (sub_dx, sub_dy) in directions.iter() {
                                if let Some(sub_chunk) =
                                    map_cache.get(x + dx + sub_dx, y + dy + sub_dy)
                                {
                                    chunks5.push(sub_chunk.clone());
                                }
                            }

                            if chunks5.len() == 5 {
                                let mut mesh_cache = mesh_cache.lock().unwrap();
                                mesh_cache
                                    .mesh_load_request
                                    .push_back(((x + dx, y + dy), chunks5));
                            }
                        }
                    }
                }
            })),
            Arc::new(Mutex::new({
                let mesh_cache = self.mesh_cache.clone();
                move |(x, y), mesh| {
                    let mut mesh_cache = mesh_cache.lock().unwrap();
                    mesh_cache.meshes.push_back(Arc::new(((x, y), mesh)));
                }
            })),
        );

        // TODO: add proper watch/unwatch
        // self.server.lock().unwrap().send(ClientMessage::WatchChunk { x: (), y: () });
    }

    pub fn set_cache_distance(&mut self, new_cache_distance: usize) {
        {
            let mut map_cache = self.map_cache.lock().unwrap();
            if map_cache.cache_distance != new_cache_distance {
                map_cache.cache_distance = new_cache_distance;
                map_cache.coords = calculate_coords(map_cache.cache_distance as f32);
                map_cache.reset();
            }
        }

        {
            if self.buffer_cache.cache_distance != new_cache_distance {
                self.buffer_cache.cache_distance = new_cache_distance;
                self.buffer_cache.coords =
                    calculate_coords(self.buffer_cache.cache_distance as f32);
                self.buffer_cache.reset();
            }
        }

        // TODO: add resize without reset
    }

    pub fn set_eye(&mut self, eye: (f32, f32)) {
        let mut map_cache = self.map_cache.lock().unwrap();
        fn upper(value: f32, old: bool) -> bool {
            let value = (value.fract() + 1.0).fract();
            if old {
                value > 0.25
            } else {
                value > 0.75
            }
        }
        let size = map_cache.cache_distance * 2 + 2;
        let old_eye_chunk_x = map_cache.x;
        let old_eye_chunk_y = map_cache.y;
        let old_eye_x_upper = map_cache.eye_x_upper;
        let old_eye_y_upper = map_cache.eye_y_upper;
        let old_min_x =
            old_eye_chunk_x - map_cache.cache_distance as i32 - if old_eye_x_upper { 0 } else { 1 };
        let old_min_y =
            old_eye_chunk_y - map_cache.cache_distance as i32 - if old_eye_y_upper { 0 } else { 1 };
        let (new_eye_x, new_eye_y) = eye;
        let new_eye_chunk_x = (new_eye_x / CHUNK_SIZE as f32).floor() as i32;
        let new_eye_chunk_y = (new_eye_y / CHUNK_SIZE as f32).floor() as i32;
        let new_eye_x_upper = upper(new_eye_x / CHUNK_SIZE as f32, old_eye_x_upper);
        let new_eye_y_upper = upper(new_eye_y / CHUNK_SIZE as f32, old_eye_y_upper);
        let new_min_x =
            new_eye_chunk_x - map_cache.cache_distance as i32 - if new_eye_x_upper { 0 } else { 1 };
        let new_min_y =
            new_eye_chunk_y - map_cache.cache_distance as i32 - if new_eye_y_upper { 0 } else { 1 };
        let new_max_x = new_min_x + size as i32 - 1;
        let new_max_y = new_min_y + size as i32 - 1;

        self.eye = eye;
        map_cache.eye_x_upper = new_eye_x_upper;
        map_cache.eye_y_upper = new_eye_y_upper;
        map_cache.x = new_eye_chunk_x;
        map_cache.y = new_eye_chunk_y;

        self.buffer_cache.eye_x_upper = new_eye_x_upper;
        self.buffer_cache.eye_y_upper = new_eye_y_upper;
        self.buffer_cache.x = new_eye_chunk_x;
        self.buffer_cache.y = new_eye_chunk_y;

        match new_min_x - old_min_x {
            0 => {}
            1 => {
                for y in 0..size {
                    let x = new_max_x.rem_euclid(size as i32) as usize;
                    map_cache.chunks[y * size + x] = None;
                    self.buffer_cache.buffers[y * size + x] = None;
                }
            }
            -1 => {
                for y in 0..size {
                    let x = new_min_x.rem_euclid(size as i32) as usize;
                    map_cache.chunks[y * size + x] = None;
                    self.buffer_cache.buffers[y * size + x] = None;
                }
            }
            _ => {
                map_cache.reset();
                self.buffer_cache.reset();
                return;
            }
        }

        match new_min_y - old_min_y {
            0 => {}
            1 => {
                for x in 0..size {
                    let y = new_max_y.rem_euclid(size as i32) as usize;
                    map_cache.chunks[y * size + x] = None;
                    self.buffer_cache.buffers[y * size + x] = None;
                }
            }
            -1 => {
                for x in 0..size {
                    let y = new_min_y.rem_euclid(size as i32) as usize;
                    map_cache.chunks[y * size + x] = None;
                    self.buffer_cache.buffers[y * size + x] = None;
                }
            }
            _ => {
                map_cache.reset();
                self.buffer_cache.reset();
                return;
            }
        }
    }

    pub fn get_available(
        &mut self,
        process: &mut dyn FnMut(&Mesh) -> (Arc<Vec<DrawCallArgs>>, Arc<Vec<DrawCallArgs>>),
    ) -> Vec<MeshBuffer> {
        self.buffer_cache
            .get_available(self.mesh_cache.clone(), process)
    }

    pub fn get_farthest_distance(&self) -> f32 {
        (self.buffer_cache.farthest_distance_sq as f32)
            .sqrt()
            .floor()
    }
}

fn calculate_coords(distance: f32) -> Vec<(i32, i32)> {
    let mut result = get_coords(distance);

    fn dst((x, y): (i32, i32)) -> i32 {
        x * x + y * y
    }
    result.sort_unstable_by(|&a, &b| dst(a).cmp(&dst(b)));

    result
}
