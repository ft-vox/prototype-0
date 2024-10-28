use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use ft_vox_prototype_0_core::TerrainWorker;
use ft_vox_prototype_0_map_core::Map;
use ft_vox_prototype_0_map_types::Chunk;

pub struct NativeTerrainWorker {
    handles: Vec<JoinHandle<()>>,
    running: Arc<Mutex<bool>>,
}

impl TerrainWorker for NativeTerrainWorker {
    fn new(
        before_load_callback: Arc<Mutex<dyn Send + Sync + FnMut() -> Option<(i32, i32, i32)>>>,
        after_load_callback: Arc<Mutex<dyn Send + Sync + FnMut((i32, i32, i32), Arc<Chunk>)>>,
    ) -> Self {
        let cpu_count = num_cpus::get_physical();
        let worker_count = (cpu_count - 1).max(1);
        let mut handles = Vec::new();
        let running = Arc::new(Mutex::new(true));

        for _ in 0..worker_count {
            handles.push(thread::spawn({
                let before_load_callback = before_load_callback.clone();
                let after_load_callback = after_load_callback.clone();
                let running = running.clone();
                move || {
                    let map = Map::new(42);
                    while *running.lock().unwrap() {
                        let option = before_load_callback.lock().unwrap()();

                        if let Some((x, y, z)) = option {
                            let chunk = map.get_chunk(x, y, z);
                            after_load_callback.lock().unwrap()((x, y, z), Arc::new(chunk));
                        } else {
                            thread::sleep(Duration::from_millis(100));
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
