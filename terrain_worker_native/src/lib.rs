use std::{
    collections::VecDeque,
    rc::Rc,
    sync::{Arc, Mutex},
    thread::{self, available_parallelism},
    time::Duration,
};

use ft_vox_prototype_0_core::{get_coords, TerrainWorker};
use ft_vox_prototype_0_map_core::Map;
use ft_vox_prototype_0_map_types::Chunk;
use ft_vox_prototype_0_util_lru_cache_arc::LRUCache as ArcLRUCache;
use ft_vox_prototype_0_util_lru_cache_rc::LRUCache;

#[derive(PartialEq, Eq, Clone, Copy)]
enum QueueItem {
    Generate((i32, i32, i32)),
}

pub struct NativeTerrainWorker {
    chunks_arc: Arc<Mutex<ArcLRUCache<(i32, i32, i32), Option<Arc<Chunk>>>>>,
    chunks_rc: LRUCache<(i32, i32, i32), Rc<Chunk>>,
    queue: Arc<Mutex<VecDeque<QueueItem>>>,
}

impl TerrainWorker for NativeTerrainWorker {
    fn new(map: Map, render_distance: f32) -> Self {
        let chunks_arc = Arc::new(Mutex::new(ArcLRUCache::new(
            get_coords(render_distance + 2.0).len() * 2,
        )));
        let chunks_rc = LRUCache::new(get_coords(render_distance + 2.0).len() * 2);
        let queue = Arc::new(Mutex::new(VecDeque::new()));

        let cpu_count = available_parallelism().unwrap().get();
        for _ in 0..(cpu_count - 1).max(1) {
            let chunks = chunks_arc.clone();
            let queue = queue.clone();
            let map = map.clone();
            thread::spawn(move || loop {
                let option = queue.lock().unwrap().pop_front();

                if let Some(QueueItem::Generate((x, y, z))) = option {
                    let chunk = map.get_chunk(x, y, z);
                    let mut chunks = chunks.lock().unwrap();
                    chunks.put((x, y, z), Some(Arc::new(chunk)));
                } else {
                    thread::sleep(Duration::from_millis(100));
                }
            });
        }

        Self {
            chunks_arc,
            chunks_rc,
            queue,
        }
    }

    fn get_available(
        &mut self,
        chunk_coords: &[(i32, i32, i32)],
    ) -> Vec<((i32, i32, i32), Rc<Chunk>)> {
        let mut result = Vec::new();
        for &chunk_coord in chunk_coords {
            if let Some(chunk) = self.chunks_rc.get(&chunk_coord) {
                result.push((chunk_coord, chunk));
            } else {
                let mut borrow = self.chunks_arc.lock().unwrap();

                if let Some(option) = borrow.get(&chunk_coord) {
                    if let Some(chunk) = option {
                        let chunk = Rc::new((*chunk).clone());
                        self.chunks_rc.put(chunk_coord, chunk.clone());
                        result.push((chunk_coord, chunk));
                    } else {
                        // loading. nothing to do here.
                    }
                } else {
                    let mut borrow = self.queue.lock().unwrap();
                    let item = QueueItem::Generate(chunk_coord);
                    if !borrow.contains(&item) {
                        borrow.push_back(item);
                    }
                }
            }
        }

        result
    }
}
