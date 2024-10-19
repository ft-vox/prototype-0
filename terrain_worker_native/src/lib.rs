use std::rc::Rc;

use ft_vox_prototype_0_core::{get_coords, TerrainWorker};
use ft_vox_prototype_0_map_core::Map;
use ft_vox_prototype_0_map_types::Chunk;
use ft_vox_prototype_0_util_lru_cache::LRUCache;

pub struct NativeTerrainWorker {
    map: Map,
    chunks: LRUCache<(i32, i32, i32), Rc<Chunk>>,
}

impl TerrainWorker for NativeTerrainWorker {
    fn new(map: Map, render_distance: f32) -> Self {
        let chunks = LRUCache::new(get_coords(render_distance).len() * 3);

        Self { map, chunks }
    }

    fn get_available(
        &mut self,
        chunk_coords: &[(i32, i32, i32)],
    ) -> Vec<((i32, i32, i32), Rc<Chunk>)> {
        let mut result = Vec::new();

        for &chunk_coord in chunk_coords {
            if let Some(chunk) = self.chunks.get(&chunk_coord) {
                result.push((chunk_coord, chunk));
            } else {
                let new_value = Rc::new(self.map.get_chunk(
                    chunk_coord.0,
                    chunk_coord.1,
                    chunk_coord.2,
                ));
                self.chunks.put(chunk_coord, new_value.clone());
                result.push((chunk_coord, new_value));
            }
        }

        result
    }
}
