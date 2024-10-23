use std::{
    collections::{HashSet, VecDeque},
    rc::Rc,
    sync::{Arc, Mutex},
    thread::{self, available_parallelism},
    time::Duration,
};

use ft_vox_prototype_0_core::{chunk_cache::ChunkCache, TerrainWorker};
use ft_vox_prototype_0_map_core::Map;
use ft_vox_prototype_0_map_types::Chunk;
use ft_vox_prototype_0_util_lru_cache_arc::LRUCache;

#[derive(PartialEq, Eq, Clone, Copy)]
enum QueueItem {
    Generate((i32, i32, i32)),
}

pub struct NativeTerrainWorker {
    chunks: Arc<Mutex<LRUCache<(i32, i32, i32), Arc<Chunk>>>>,
    chunk_cache: ChunkCache,
    queue: Arc<Mutex<VecDeque<QueueItem>>>,
    is_loading: Arc<Mutex<HashSet<(i32, i32, i32)>>>,
}

impl TerrainWorker for NativeTerrainWorker {
    fn new(cache_distance: usize, eye: (f32, f32, f32)) -> Self {
        let cpu_count = available_parallelism().unwrap().get();

        let chunks = Arc::new(Mutex::new(LRUCache::new(cpu_count * 420)));
        let chunk_cache = ChunkCache::new(cache_distance, eye);
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let is_loading = Arc::new(Mutex::new(HashSet::new()));

        for _ in 0..(cpu_count - 1).max(1) {
            let chunks = chunks.clone();
            let queue = queue.clone();
            let is_loading = is_loading.clone();
            thread::spawn(move || {
                let map = Map::new(42);
                loop {
                    let option = queue.lock().unwrap().pop_front();

                    if let Some(QueueItem::Generate((x, y, z))) = option {
                        let chunk = map.get_chunk(x, y, z);
                        let mut chunks = chunks.lock().unwrap();
                        chunks.put((x, y, z), Arc::new(chunk));
                        is_loading.lock().unwrap().remove(&(x, y, z));
                    } else {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            });
        }

        Self {
            chunks,
            chunk_cache,
            queue,
            is_loading,
        }
    }

    fn get_available(
        &mut self,
        cache_distance: usize,
        (x, y, z): (f32, f32, f32),
    ) -> Vec<((i32, i32, i32), Rc<Chunk>)> {
        let mut result = Vec::new();
        self.chunk_cache.set_cache_distance(cache_distance);
        self.chunk_cache.set_eye((x, y, z));
        for (x, y, z) in self.chunk_cache.coords().clone() {
            if let Some(chunk) = self.chunk_cache.get(x, y, z) {
                result.push(((x, y, z), chunk));
            } else {
                let mut chunks = self.chunks.lock().unwrap();

                if let Some(chunk) = chunks.get(&(x, y, z)) {
                    let chunk = Rc::new((*chunk).clone());
                    self.chunk_cache.set(x, y, z, Some(chunk.clone()));
                    result.push(((x, y, z), chunk));
                    println!("Loaded {}, {}, {}", x, y, z);
                } else if !self.is_loading.lock().unwrap().contains(&(x, y, z)) {
                    self.is_loading.lock().unwrap().insert((x, y, z));
                    self.queue
                        .lock()
                        .unwrap()
                        .push_back(QueueItem::Generate((x, y, z)));
                }
            }
        }

        result
    }
}
