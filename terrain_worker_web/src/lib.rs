use std::{cell::RefCell, fmt::format, rc::Rc};

use ft_vox_prototype_0_core::{get_coords, TerrainWorker};
use ft_vox_prototype_0_map_core::Map;
use ft_vox_prototype_0_map_types::Chunk;
use ft_vox_prototype_0_util_lru_cache::LRUCache;
use js_sys::wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    console, window, File, FileSystemDirectoryHandle, FileSystemFileHandle, MessageEvent, Window,
    Worker,
};

pub struct WebTerrainWorker {
    map: Map,
    chunks: Rc<RefCell<LRUCache<(i32, i32, i32), Option<Option<Rc<Chunk>>>>>>,
    to_load: Rc<RefCell<Vec<(i32, i32, i32)>>>,
    worker: Worker,
}

impl TerrainWorker for WebTerrainWorker {
    fn new(map: Map, render_distance: f32) -> Self {
        let chunks = Rc::new(RefCell::new(LRUCache::new(
            get_coords(render_distance).len() * 3,
        )));
        let to_load = Rc::new(RefCell::new(Vec::new()));
        let worker = Worker::new("terrain-worker-main.js").unwrap();

        {
            let chunks = chunks.clone();
            let worker_clone = worker.clone();
            let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
                let data = event
                    .data()
                    .as_string()
                    .unwrap()
                    .split(',')
                    .flat_map(&str::parse::<i32>)
                    .collect::<Vec<_>>();

                wasm_bindgen_futures::spawn_local(load_map(
                    worker_clone.clone(),
                    chunks.clone(),
                    (data[0], data[1], data[2]),
                    false,
                ));
            }) as Box<dyn FnMut(MessageEvent)>);

            worker.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget();
        }

        Self {
            map,
            chunks,
            to_load,
            worker,
        }
    }

    fn get_available(
        &mut self,
        chunk_coords: &[(i32, i32, i32)],
    ) -> Vec<((i32, i32, i32), Rc<Chunk>)> {
        let mut result = Vec::new();
        *self.to_load.borrow_mut() = Vec::new();

        for &chunk_coord in chunk_coords {
            if let Some(option) = self.chunks.borrow_mut().get(&chunk_coord) {
                if let Some(option) = option {
                    if let Some(chunk) = option {
                        result.push((chunk_coord, chunk));
                    } // else loading
                } else {
                    self.chunks.borrow_mut().put(chunk_coord, Some(None));
                    wasm_bindgen_futures::spawn_local(load_map(
                        self.worker.clone(),
                        self.chunks.clone(),
                        chunk_coord,
                        true,
                    ));
                }
            } else {
                self.chunks.borrow_mut().put(chunk_coord, None);
                self.to_load.borrow_mut().push(chunk_coord);
            }
        }

        result
    }
}

async fn load_map(
    worker: Worker,
    chunks: Rc<RefCell<LRUCache<(i32, i32, i32), Option<Option<Rc<Chunk>>>>>>,
    (x, y, z): (i32, i32, i32),
    trigger: bool,
) {
    let directory: FileSystemDirectoryHandle =
        JsFuture::from(window().unwrap().navigator().storage().get_directory())
            .await
            .unwrap()
            .dyn_into()
            .unwrap();

    let file_name = format!("{}_{}_{}.chunk", x, y, z);

    let file_handle: FileSystemFileHandle = JsFuture::from(directory.get_file_handle(&file_name))
        .await
        .unwrap()
        .dyn_into()
        .unwrap();

    if let Ok(file) = JsFuture::from(file_handle.get_file())
        .await
        .unwrap()
        .dyn_into::<File>()
    {
        let file_contents = JsFuture::from(file.array_buffer()).await.unwrap();
        console::log_2(
            &JsValue::from_str(format!("Loaded {}, {}, {}", x, y, z).as_str()),
            &file_contents,
        );
    } else if trigger {
        worker
            .post_message(&JsValue::from_str(format!("{},{},{}", x, y, z).as_str()))
            .unwrap();
    }
}
