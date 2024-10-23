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

#[derive(PartialEq, Eq, Clone, Copy)]
enum QueueItem {
    Generate((i32, i32, i32)),
}

type Position = (i32, i32, i32);
type RequestQueueItem = (Position, Arc<Chunk>);

pub struct NativeTerrainWorker {
    chunk_cache: ChunkCache,
    request_queue: Arc<Mutex<VecDeque<QueueItem>>>,
    result_queue: Arc<Mutex<VecDeque<RequestQueueItem>>>,
    is_loading: Arc<Mutex<HashSet<Position>>>,
}

impl TerrainWorker for NativeTerrainWorker {
    fn new(cache_distance: usize, eye: (f32, f32, f32)) -> Self {
        let cpu_count = available_parallelism().unwrap().get();

        let chunk_cache = ChunkCache::new(cache_distance, eye);
        let request_queue = Arc::new(Mutex::new(VecDeque::new()));
        let result_queue = Arc::new(Mutex::new(VecDeque::new()));
        let is_loading = Arc::new(Mutex::new(HashSet::new()));

        for _ in 0..(cpu_count - 1).max(1) {
            let result_queue = result_queue.clone();
            let request_queue = request_queue.clone();
            let is_loading = is_loading.clone();
            thread::spawn(move || {
                let map = Map::new(42);
                loop {
                    let option = request_queue.lock().unwrap().pop_front();

                    if let Some(QueueItem::Generate((x, y, z))) = option {
                        let chunk = map.get_chunk(x, y, z);
                        let mut result_queue = result_queue.lock().unwrap();
                        result_queue.push_back(((x, y, z), Arc::new(chunk)));
                        is_loading.lock().unwrap().remove(&(x, y, z));
                    } else {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            });
        }

        Self {
            chunk_cache,
            request_queue,
            result_queue,
            is_loading,
        }
    }

    fn get_available(
        &mut self,
        cache_distance: usize,
        (x, y, z): (f32, f32, f32),
    ) -> Vec<((i32, i32, i32), Rc<Chunk>)> {
        self.chunk_cache.set_cache_distance(cache_distance + 1);
        self.chunk_cache.set_eye((x, y, z));
        {
            let mut result_queue = self.result_queue.lock().unwrap();
            while let Some(((x, y, z), chunk)) = result_queue.pop_front() {
                let chunk = Rc::new((*chunk).clone());
                self.chunk_cache.set(x, y, z, Some(chunk));
            }
        }

        let mut result = Vec::new();
        for (x, y, z) in self.chunk_cache.coords().clone() {
            if let Some(chunk) = self.chunk_cache.get(x, y, z) {
                result.push(((x, y, z), chunk));
            } else if !self.is_loading.lock().unwrap().contains(&(x, y, z)) {
                self.is_loading.lock().unwrap().insert((x, y, z));
                self.request_queue
                    .lock()
                    .unwrap()
                    .push_back(QueueItem::Generate((x, y, z)));
            }
        }

        result
    }
}
