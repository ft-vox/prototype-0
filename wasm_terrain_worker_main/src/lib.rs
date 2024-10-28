use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::DedicatedWorkerGlobalScope;
use web_sys::MessageEvent;
use web_sys::Worker;

#[wasm_bindgen]
pub fn start_worker() {
    let global_scope: DedicatedWorkerGlobalScope = js_sys::global().dyn_into().unwrap();
    let workers = Rc::new(
        (0..(global_scope.navigator().hardware_concurrency() as usize - 1).max(1))
            .map(|_| Worker::new("terrain-worker-sub.js").unwrap())
            .collect::<Vec<_>>(),
    );

    // global scope
    {
        let workers = workers.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            let data = event
                .data()
                .as_string()
                .unwrap()
                .split(',')
                .flat_map(&str::parse::<i32>)
                .collect::<Vec<_>>();
            workers[data[0] as usize]
                .post_message(&JsValue::from_str(&format!(
                    "{},{},{}",
                    data[1], data[2], data[3]
                )))
                .unwrap();
        }) as Box<dyn FnMut(MessageEvent)>);

        global_scope.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
    }

    // per worker
    for (i, worker) in workers.iter().enumerate() {
        let global_scope = global_scope.clone();

        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            let data = event.data();

            if data.as_string().unwrap().starts_with("request") {
                global_scope
                    .post_message(&JsValue::from_str(&format!("request,{}", i)))
                    .unwrap();
            } else {
                global_scope.post_message(&data).unwrap();
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        worker.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
    }
}
