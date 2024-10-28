use std::rc::Rc;

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

    {
        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            let data = event.data();

            wasm_bindgen_futures::spawn_local(process_job(map.clone(), data));
        }) as Box<dyn FnMut(MessageEvent)>);

        global_scope.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
    }

    global_scope
        .post_message(&JsValue::from_str("request"))
        .unwrap();
}

async fn process_job(map: Rc<Map>, data: JsValue) {
    let global_scope: DedicatedWorkerGlobalScope = js_sys::global().dyn_into().unwrap();

    let [x, y, z] = data
        .as_string()
        .unwrap()
        .split(',')
        .flat_map(&str::parse::<i32>)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    if x == i32::MIN && y == i32::MIN && z == i32::MIN {
        delay_ms(100).await
    } else {
        generate_map(map, (x, y, z)).await;
    }
    global_scope.post_message(&data).unwrap();
    global_scope
        .post_message(&JsValue::from_str("request"))
        .unwrap();
}

async fn generate_map(map: Rc<Map>, (x, y, z): (i32, i32, i32)) {
    let global_scope: DedicatedWorkerGlobalScope = js_sys::global().dyn_into().unwrap();

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
                "worker: Failed to get access ({}, {}, {})",
                x, y, z
            )),
            &access_result.unwrap_err(),
        );
        return;
    }
    let access: FileSystemSyncAccessHandle = access_result.unwrap().dyn_into().unwrap();

    access
        .write_with_u8_array(&map.get_chunk(x, y, z).to_u8_vec())
        .unwrap();
    access.flush().unwrap();
    access.close();
}

async fn delay_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        js_sys::global()
            .dyn_into::<DedicatedWorkerGlobalScope>()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
            .unwrap();
    });
    JsFuture::from(promise).await.unwrap();
}
