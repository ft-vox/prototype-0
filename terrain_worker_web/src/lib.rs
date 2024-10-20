use std::{cell::RefCell, rc::Rc};

use ft_vox_prototype_0_core::{get_coords, TerrainWorker};
use ft_vox_prototype_0_map_core::Map;
use ft_vox_prototype_0_map_types::{Chunk, CHUNK_SIZE};
use ft_vox_prototype_0_util_lru_cache::LRUCache;
use js_sys::{
    wasm_bindgen::{prelude::Closure, JsCast, JsValue},
    Uint8Array,
};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    console, window, File, FileSystemDirectoryHandle, FileSystemFileHandle, MessageEvent, Worker,
};

pub struct WebTerrainWorker {
    chunks: Rc<RefCell<LRUCache<(i32, i32, i32), Option<Rc<Chunk>>>>>,
    worker: Worker,
    worker_ready: Rc<RefCell<bool>>,
}

impl TerrainWorker for WebTerrainWorker {
    fn new(_map: Map, render_distance: f32) -> Self {
        let chunks = Rc::new(RefCell::new(LRUCache::new(
            get_coords(render_distance + 2.0).len() * 2,
        )));
        let worker = Worker::new("terrain-worker-main.js").unwrap();
        let worker_ready = Rc::new(RefCell::new(false));

        {
            let chunks = chunks.clone();
            let worker_ready = worker_ready.clone();
            let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
                let data = event.data().as_string().unwrap();
                match data.as_str() {
                    "init" => {
                        *worker_ready.borrow_mut() = true;
                    }
                    _ => {
                        let [x, y, z] = data
                            .split(',')
                            .flat_map(&str::parse::<i32>)
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap();

                        wasm_bindgen_futures::spawn_local(load_map(chunks.clone(), (x, y, z)));
                    }
                }
            }) as Box<dyn FnMut(MessageEvent)>);

            worker.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget();
        }

        Self {
            chunks,
            worker,
            worker_ready,
        }
    }

    fn get_available(
        &mut self,
        chunk_coords: &[(i32, i32, i32)],
    ) -> Vec<((i32, i32, i32), Rc<Chunk>)> {
        let mut result = Vec::new();
        let mut borrow = self.chunks.borrow_mut();

        for &chunk_coord in chunk_coords {
            if let Some(option) = borrow.get(&chunk_coord) {
                if let Some(chunk) = option {
                    result.push((chunk_coord, chunk));
                } else {
                    // loading. nothing to do here.
                }
            } else if *self.worker_ready.borrow() {
                let (x, y, z) = chunk_coord;
                borrow.put(chunk_coord, None);
                self.worker
                    .post_message(&JsValue::from_str(format!("{},{},{}", x, y, z).as_str()))
                    .unwrap();
            }
        }

        result
    }
}

async fn load_map(
    chunks: Rc<RefCell<LRUCache<(i32, i32, i32), Option<Rc<Chunk>>>>>,
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
            chunks.borrow_mut().put((x, y, z), Some(Rc::new(chunk)));
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
