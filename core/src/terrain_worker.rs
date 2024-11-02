use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::vertex::*;
use ft_vox_prototype_0_map_core::Map;
use ft_vox_prototype_0_map_types::Chunk;

pub enum TerrainWorkerJob {
    Map((i32, i32, i32)),
    Mesh {
        position: (i32, i32, i32),
        zero: Arc<Chunk>,
        positive_x: Arc<Chunk>,
        negative_x: Arc<Chunk>,
        positive_y: Arc<Chunk>,
        negative_y: Arc<Chunk>,
        positive_z: Arc<Chunk>,
        negative_z: Arc<Chunk>,
    },
}

pub struct TerrainWorker {
    handles: Vec<JoinHandle<()>>,
    running: Arc<Mutex<bool>>,
}

impl TerrainWorker {
    pub fn new(
        job_callback: Arc<Mutex<dyn Send + Sync + FnMut() -> Option<TerrainWorkerJob>>>,
        chunk_callback: Arc<Mutex<dyn Send + Sync + FnMut((i32, i32, i32), Arc<Chunk>)>>,
        mesh_callback: Arc<
            Mutex<dyn Send + Sync + FnMut((i32, i32, i32), (Vec<Vertex>, Vec<u16>))>,
        >,
    ) -> Self {
        let cpu_count = num_cpus::get_physical();
        let worker_count = (cpu_count - 1).max(1);
        let mut handles = Vec::new();
        let running = Arc::new(Mutex::new(true));

        for _ in 0..worker_count {
            handles.push(thread::spawn({
                let job_callback = job_callback.clone();
                let chunk_callback = chunk_callback.clone();
                let mesh_callback = mesh_callback.clone();
                let running = running.clone();
                move || {
                    let map = Map::new(42);
                    while *running.lock().unwrap() {
                        let option = job_callback.lock().unwrap()();
                        if let Some(job) = option {
                            match job {
                                TerrainWorkerJob::Map((x, y, z)) => {
                                    let chunk = map.get_chunk(x, y, z);
                                    chunk_callback.lock().unwrap()((x, y, z), Arc::new(chunk));
                                }
                                TerrainWorkerJob::Mesh {
                                    position: (x, y, z),
                                    zero,
                                    positive_x,
                                    negative_x,
                                    positive_y,
                                    negative_y,
                                    positive_z,
                                    negative_z,
                                } => {
                                    let mesh = create_vertices_for_chunk(
                                        &zero,
                                        x,
                                        y,
                                        z,
                                        &positive_x,
                                        &negative_x,
                                        &positive_y,
                                        &negative_y,
                                        &positive_z,
                                        &negative_z,
                                    );
                                    mesh_callback.lock().unwrap()((x, y, z), mesh);
                                }
                            }
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

impl Drop for TerrainWorker {
    fn drop(&mut self) {
        *self.running.lock().unwrap() = false;
        for handle in self.handles.drain(..) {
            handle.join().expect("Thread join failed");
        }
    }
}
