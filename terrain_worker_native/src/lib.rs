use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use ft_vox_prototype_0_core::vertex::*;
use ft_vox_prototype_0_core::TerrainWorker;
use ft_vox_prototype_0_map_core::Map;
use ft_vox_prototype_0_map_types::Chunk;

pub struct NativeTerrainWorker {
    handles: Vec<JoinHandle<()>>,
    running: Arc<Mutex<bool>>,
}

impl TerrainWorker for NativeTerrainWorker {
    fn new(
        before_chunk_callback: Arc<Mutex<dyn Send + Sync + FnMut() -> Option<(i32, i32, i32)>>>,
        after_chunk_callback: Arc<Mutex<dyn Send + Sync + FnMut((i32, i32, i32), Arc<Chunk>)>>,
        before_mesh_callback: Arc<
            Mutex<dyn Send + Sync + FnMut() -> Option<((i32, i32, i32), Vec<Arc<Chunk>>)>>,
        >,
        after_mesh_callback: Arc<
            Mutex<dyn Send + Sync + FnMut((i32, i32, i32), Arc<(Vec<Vertex>, Vec<u16>)>)>,
        >,
    ) -> Self {
        let cpu_count = num_cpus::get_physical();
        let worker_count = (cpu_count - 1).max(1);
        let mut handles = Vec::new();
        let running = Arc::new(Mutex::new(true));

        for _ in 0..worker_count {
            handles.push(thread::spawn({
                let before_chunk_callback = before_chunk_callback.clone();
                let after_chunk_callback = after_chunk_callback.clone();

                let before_mesh_callback = before_mesh_callback.clone();
                let after_mesh_callback = after_mesh_callback.clone();
                let running = running.clone();
                move || {
                    let map = Map::new(42);
                    while *running.lock().unwrap() {
                        let option = before_mesh_callback.lock().unwrap()();
                        if let Some(chunks) = option {
                            let (x, y, z) = chunks.0;
                            let chunks7 = chunks.1;
                            let mesh = create_vertices_for_chunk(
                                &chunks7[0],
                                x,
                                y,
                                z,
                                &chunks7[1],
                                &chunks7[2],
                                &chunks7[3],
                                &chunks7[4],
                                &chunks7[5],
                                &chunks7[6],
                            );
                            after_mesh_callback.lock().unwrap()((x, y, z), Arc::new(mesh));
                        }

                        let option = before_chunk_callback.lock().unwrap()();

                        if let Some((x, y, z)) = option {
                            let chunk = map.get_chunk(x, y, z);
                            after_chunk_callback.lock().unwrap()((x, y, z), Arc::new(chunk));
                        } else {
                            thread::sleep(Duration::from_millis(10));
                        }
                    }
                }
            }));
        }

        Self { handles, running }
    }
}

impl Drop for NativeTerrainWorker {
    fn drop(&mut self) {
        *self.running.lock().unwrap() = false;
        for handle in self.handles.drain(..) {
            handle.join().expect("Thread join failed");
        }
    }
}
