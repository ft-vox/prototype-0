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

    let global_scope_clone = global_scope.clone();
    let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
        let data = event.data();

        let global_scope_clone_clone = global_scope_clone.clone();
        wasm_bindgen_futures::spawn_local(do_something(global_scope_clone_clone, data));
    }) as Box<dyn FnMut(MessageEvent)>);

    global_scope.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();
}

async fn do_something(global_scope: DedicatedWorkerGlobalScope, data: JsValue) {
    console::log_2(&JsValue::from_str("Worker received from main:"), &data);

    global_scope.post_message(&data).unwrap();

    // let directory: FileSystemDirectoryHandle =
    //     JsFuture::from(global_scope.navigator().storage().get_directory())
    //         .await
    //         .unwrap()
    //         .dyn_into()
    //         .unwrap();

    // let file_name = data.as_string().unwrap();

    // let option_with_create = FileSystemGetFileOptions::new();
    // option_with_create.set_create(true);

    // let file: FileSystemFileHandle =
    //     JsFuture::from(directory.get_file_handle_with_options(&file_name, &option_with_create))
    //         .await
    //         .unwrap()
    //         .dyn_into()
    //         .unwrap();

    // let access: FileSystemSyncAccessHandle = JsFuture::from(file.create_sync_access_handle())
    //     .await
    //     .unwrap()
    //     .dyn_into()
    //     .unwrap();

    // access
    //     .write_with_u8_array("Hello world".as_bytes())
    //     .unwrap();
    // access.flush().unwrap();
    // access.close();

    // global_scope
    //     .post_message(&JsValue::from_str(&file_name))
    //     .unwrap();
}
