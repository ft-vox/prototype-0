use std::{cell::RefCell, collections::HashSet, rc::Rc};

use ft_vox_prototype_0_core::{chunk_cache::ChunkCache, TerrainWorker};
use ft_vox_prototype_0_map_types::{Chunk, CHUNK_SIZE};
use js_sys::{
    wasm_bindgen::{prelude::Closure, JsCast, JsValue},
    Uint8Array,
};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    console, window, File, FileSystemDirectoryHandle, FileSystemFileHandle, MessageEvent, Worker,
};

pub struct WebTerrainWorker {
    worker: Worker,
    worker_ready: Rc<RefCell<bool>>,
    chunk_cache: Rc<RefCell<ChunkCache>>,
    is_loading: HashSet<(i32, i32, i32)>,
}

impl TerrainWorker for WebTerrainWorker {
    fn new(cache_distance: usize, eye: (f32, f32, f32)) -> Self {
        let chunk_cache = Rc::new(RefCell::new(ChunkCache::new(cache_distance, eye)));
        let worker = Worker::new("terrain-worker-main.js").unwrap();
        let worker_ready = Rc::new(RefCell::new(false));

        {
            let chunk_cache = chunk_cache.clone();
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

                        wasm_bindgen_futures::spawn_local(load_map(chunk_cache.clone(), (x, y, z)));
                    }
                }
            }) as Box<dyn FnMut(MessageEvent)>);

            worker.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget();
        }

        Self {
            worker,
            worker_ready,
            chunk_cache,
            is_loading: HashSet::new(),
        }
    }

    fn get_available(
        &mut self,
        cache_distance: usize,
        (x, y, z): (f32, f32, f32),
    ) -> Vec<((i32, i32, i32), Rc<Chunk>)> {
        let mut result = Vec::new();
        let mut borrow = self.chunk_cache.borrow_mut();
        borrow.set_cache_distance(cache_distance);
        borrow.set_eye((x, y, z));

        for (x, y, z) in borrow.coords().clone() {
            if let Some(chunk) = borrow.get(x, y, z) {
                result.push(((x, y, z), chunk));
            } else if *self.worker_ready.borrow() && !self.is_loading.contains(&(x, y, z)) {
                self.is_loading.insert((x, y, z));
                self.worker
                    .post_message(&JsValue::from_str(format!("{},{},{}", x, y, z).as_str()))
                    .unwrap();
            }
        }

        result
    }
}

async fn load_map(chunks: Rc<RefCell<ChunkCache>>, (x, y, z): (i32, i32, i32)) {
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
            chunks.borrow_mut().set(x, y, z, Some(Rc::new(chunk)));
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
