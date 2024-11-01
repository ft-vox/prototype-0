use std::sync::{Arc, Mutex};

use ft_vox_prototype_0_core::{
    vertex::{create_vertices_for_chunk, Vertex},
    TerrainWorker, TerrainWorkerJob,
};
use ft_vox_prototype_0_map_types::{Chunk, CHUNK_SIZE};
use js_sys::{
    wasm_bindgen::{prelude::Closure, JsCast, JsValue},
    Uint8Array,
};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    console, window, File, FileSystemDirectoryHandle, FileSystemFileHandle, MessageEvent, Window,
    Worker,
};

pub struct WebTerrainWorker {}

impl TerrainWorker for WebTerrainWorker {
    fn new(
        job_callback: Arc<Mutex<dyn Send + Sync + FnMut() -> Option<TerrainWorkerJob>>>,
        chunk_callback: Arc<Mutex<dyn Send + Sync + FnMut((i32, i32, i32), Arc<Chunk>)>>,
        mesh_callback: Arc<
            Mutex<dyn Send + Sync + FnMut((i32, i32, i32), (Vec<Vertex>, Vec<u16>))>,
        >,
    ) -> Self {
        let worker = Worker::new("terrain-worker-main.js").unwrap();

        {
            let worker = worker.clone();
            let worker_clone = worker.clone();
            let job_callback = job_callback.clone();
            let mesh_callback = mesh_callback.clone();
            let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
                let data = event.data().as_string().unwrap();

                if data.starts_with("request,") {
                    let [i] = data
                        .split(',')
                        .flat_map(&str::parse::<usize>)
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap();

                    if let Some(job) = job_callback.lock().unwrap()() {
                        match job {
                            TerrainWorkerJob::Map((x, y, z)) => {
                                worker
                                    .post_message(&JsValue::from_str(&format!(
                                        "{},{},{},{}",
                                        i, x, y, z
                                    )))
                                    .unwrap();
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
                                worker
                                    .post_message(&JsValue::from_str(&format!(
                                        "{},-2147483648,-2147483648,-2147483648",
                                        i
                                    )))
                                    .unwrap();
                            }
                        }
                    } else {
                        worker
                            .post_message(&JsValue::from_str(&format!(
                                "{},-2147483648,-2147483648,-2147483648",
                                i
                            )))
                            .unwrap();
                    }
                } else {
                    let [x, y, z] = data
                        .split(',')
                        .flat_map(&str::parse::<i32>)
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap();

                    wasm_bindgen_futures::spawn_local(load_map(chunk_callback.clone(), (x, y, z)));
                }
            }) as Box<dyn FnMut(MessageEvent)>);

            worker_clone.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget();
        }

        Self {}
    }
}

async fn load_map(
    after_load_callback: Arc<Mutex<dyn Send + Sync + FnMut((i32, i32, i32), Arc<Chunk>)>>,
    (x, y, z): (i32, i32, i32),
) {
    let directory: FileSystemDirectoryHandle =
        JsFuture::from(window().unwrap().navigator().storage().get_directory())
            .await
            .unwrap()
            .dyn_into()
            .unwrap();

    let file_name = format!("0_{}_{}_{}.chunk", x, y, z);

    let file_handle_result = JsFuture::from(directory.get_file_handle(&file_name)).await;
    if let Ok(file_handle) = file_handle_result {
        let file_handle: FileSystemFileHandle = file_handle.dyn_into().unwrap();
        let file: File = JsFuture::from(file_handle.get_file())
            .await
            .unwrap()
            .dyn_into()
            .unwrap();
        let file_contents =
            Uint8Array::new(&JsFuture::from(file.array_buffer()).await.unwrap()).to_vec();
        if file_contents.len() == CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE {
            let chunk = Chunk::from_u8_vec(&file_contents);
            after_load_callback.lock().unwrap()((x, y, z), Arc::new(chunk));
        } else {
            console::error_1(&JsValue::from_str(&format!(
                "File corrupted ({}, {}, {})\nrun `(async () => {{ await (await navigator.storage.getDirectory()).remove({{ recursive: true }}); }})()` and reload.",
                x, y, z
            )));
        }
    } else {
        console::error_1(&JsValue::from_str("wtf"));
    }
}
