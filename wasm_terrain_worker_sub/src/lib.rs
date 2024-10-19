use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use ft_vox_prototype_0_map_core::Map;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::console;
use web_sys::DedicatedWorkerGlobalScope;
use web_sys::FileSystemDirectoryHandle;
use web_sys::FileSystemFileHandle;
use web_sys::FileSystemGetFileOptions;
use web_sys::FileSystemSyncAccessHandle;
use web_sys::MessageEvent;

#[wasm_bindgen]
pub fn start_worker() {
    let global_scope: DedicatedWorkerGlobalScope = js_sys::global().dyn_into().unwrap();
    let map = Rc::new(Map::new(42));
    let worker_id = Rc::new(RefCell::new(42));

    global_scope
        .post_message(&JsValue::from_str("init"))
        .unwrap();

    {
        let map = map.clone();
        let worker_id = worker_id.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            let data = event.data();

            wasm_bindgen_futures::spawn_local(process_job(worker_id.clone(), map.clone(), data));
        }) as Box<dyn FnMut(MessageEvent)>);

        global_scope.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
    }
}

async fn process_job(worker_id: Rc<RefCell<i32>>, map: Rc<Map>, data: JsValue) {
    let global_scope: DedicatedWorkerGlobalScope = js_sys::global().dyn_into().unwrap();

    let args = data
        .as_string()
        .unwrap()
        .split(':')
        .flat_map(String::from_str)
        .collect::<Vec<_>>();

    match args[0].as_str() {
        "init" => {
            *worker_id.borrow_mut() = args[1].parse().unwrap();
        }
        "generate" => {
            let [x, y, z] = args[1]
                .split(',')
                .flat_map(&str::parse::<i32>)
                .collect::<Vec<_>>()
                .try_into()
                .expect("Invalid message given");
            generate_map(worker_id, map, (x, y, z)).await;
            global_scope
                .post_message(&JsValue::from_str(&format!("generate:{},{},{}", x, y, z)))
                .unwrap();
        }
        _ => panic!("Invalid message given"),
    }
}

async fn generate_map(worker_id: Rc<RefCell<i32>>, map: Rc<Map>, (x, y, z): (i32, i32, i32)) {
    let global_scope: DedicatedWorkerGlobalScope = js_sys::global().dyn_into().unwrap();

    console::log_1(&JsValue::from_str(&format!(
        "worker {}: generating map ({}, {}, {})",
        *worker_id.borrow(),
        x,
        y,
        z
    )));

    let directory: FileSystemDirectoryHandle =
        JsFuture::from(global_scope.navigator().storage().get_directory())
            .await
            .unwrap()
            .dyn_into()
            .unwrap();

    let file_name = format!("0_{}_{}_{}.chunk", x, y, z);

    if JsFuture::from(directory.get_file_handle(&file_name))
        .await
        .is_ok()
    {
        return;
    }

    let option_with_create = FileSystemGetFileOptions::new();
    option_with_create.set_create(true);

    let file: FileSystemFileHandle =
        JsFuture::from(directory.get_file_handle_with_options(&file_name, &option_with_create))
            .await
            .unwrap()
            .dyn_into()
            .unwrap();

    let access_result = JsFuture::from(file.create_sync_access_handle()).await;
    if access_result.is_err() {
        console::error_2(
            &JsValue::from_str(&format!(
                "worker {}: Failed to get access ({}, {}, {})",
                *worker_id.borrow(),
                x,
                y,
                z
            )),
            &access_result.unwrap_err(),
        );
        return;
    }
    let access: FileSystemSyncAccessHandle = access_result.unwrap().dyn_into().unwrap();

    console::log_1(&JsValue::from_str(&format!(
        "worker {}: 2",
        *worker_id.borrow(),
    )));

    access
        .write_with_u8_array(&map.get_chunk(x, y, z).to_u8_vec())
        .unwrap();
    access.flush().unwrap();
    access.close();
}
