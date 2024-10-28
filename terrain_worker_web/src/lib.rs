use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use ft_vox_prototype_0_core::TerrainWorker;
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
        before_load_callback: Arc<Mutex<dyn Send + Sync + FnMut() -> Option<(i32, i32, i32)>>>,
        after_load_callback: Arc<Mutex<dyn Send + Sync + FnMut((i32, i32, i32), Arc<Chunk>)>>,
    ) -> Self {
        let worker = Worker::new("terrain-worker-main.js").unwrap();

        {
            let worker = worker.clone();
            let worker_clone = worker.clone();
            let before_load_callback = before_load_callback.clone();
            let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
                let data = event.data().as_string().unwrap();

                if data.starts_with("request,") {
                    let [i] = data
                        .split(',')
                        .flat_map(&str::parse::<usize>)
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap();

                    if let Some((x, y, z)) = before_load_callback.lock().unwrap()() {
                        worker
                            .post_message(&JsValue::from_str(&format!("{},{},{},{}", i, x, y, z)))
                            .unwrap();
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

                    wasm_bindgen_futures::spawn_local(load_map(
                        after_load_callback.clone(),
                        (x, y, z),
                    ));
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
