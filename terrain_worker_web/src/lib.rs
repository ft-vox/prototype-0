use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use ft_vox_prototype_0_core::{
    vertex::{create_vertices_for_chunk, Vertex},
    TerrainWorker, TerrainWorkerJob,
};
use ft_vox_prototype_0_map_core::Map;
use ft_vox_prototype_0_map_types::Chunk;
use js_sys::wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{console, Window};

pub struct WebTerrainWorker {
    running: Rc<RefCell<bool>>,
    job_callback: Arc<Mutex<dyn Send + Sync + FnMut() -> Option<TerrainWorkerJob>>>,
    chunk_callback: Arc<Mutex<dyn Send + Sync + FnMut((i32, i32, i32), Arc<Chunk>)>>,
    mesh_callback: Arc<Mutex<dyn Send + Sync + FnMut((i32, i32, i32), (Vec<Vertex>, Vec<u16>))>>,
}

impl TerrainWorker for WebTerrainWorker {
    fn new(
        job_callback: Arc<Mutex<dyn Send + Sync + FnMut() -> Option<TerrainWorkerJob>>>,
        chunk_callback: Arc<Mutex<dyn Send + Sync + FnMut((i32, i32, i32), Arc<Chunk>)>>,
        mesh_callback: Arc<
            Mutex<dyn Send + Sync + FnMut((i32, i32, i32), (Vec<Vertex>, Vec<u16>))>,
        >,
    ) -> Self {
        let running = Rc::new(RefCell::new(true));
        let result = Self {
            running,
            job_callback,
            chunk_callback,
            mesh_callback,
        };
        result.init();
        result
    }
}

impl WebTerrainWorker {
    fn init(&self) {
        spawn_local(Self::start(
            self.running.clone(),
            self.job_callback.clone(),
            self.chunk_callback.clone(),
            self.mesh_callback.clone(),
        ));
    }

    async fn start(
        running: Rc<RefCell<bool>>,
        job_callback: Arc<Mutex<dyn Send + Sync + FnMut() -> Option<TerrainWorkerJob>>>,
        chunk_callback: Arc<Mutex<dyn Send + Sync + FnMut((i32, i32, i32), Arc<Chunk>)>>,
        mesh_callback: Arc<
            Mutex<dyn Send + Sync + FnMut((i32, i32, i32), (Vec<Vertex>, Vec<u16>))>,
        >,
    ) {
        let map = Map::new(42);
        while *running.borrow() {
            console::log_1(&JsValue::from_str("Hello world!"));
            Self::delay_ms(10).await;
            for _ in 0..42 {
                if let Some(job) = job_callback.lock().unwrap()() {
                    match job {
                        TerrainWorkerJob::Map((x, y, z)) => {
                            chunk_callback.lock().unwrap()(
                                (x, y, z),
                                Arc::new(map.get_chunk(x, y, z)),
                            );
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
                            mesh_callback.lock().unwrap()(
                                (x, y, z),
                                create_vertices_for_chunk(
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
                                ),
                            );
                        }
                    }
                }
            }
        }
    }

    async fn delay_ms(ms: i32) {
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            js_sys::global()
                .dyn_into::<Window>()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
                .unwrap();
        });
        JsFuture::from(promise).await.unwrap();
    }
}

impl Drop for WebTerrainWorker {
    fn drop(&mut self) {
        *self.running.borrow_mut() = false;
    }
}
