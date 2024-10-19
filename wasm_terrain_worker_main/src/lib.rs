use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::str::FromStr;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::console;
use web_sys::DedicatedWorkerGlobalScope;
use web_sys::MessageEvent;
use web_sys::Worker;

#[derive(PartialEq, Eq, Clone, Copy)]
enum QueueItem {
    Init,
    Generate((i32, i32, i32)),
}

#[derive(PartialEq, Eq)]
enum WorkerState {
    NotReady,
    Ready(Option<QueueItem>),
}

#[wasm_bindgen]
pub fn start_worker() {
    let global_scope: DedicatedWorkerGlobalScope = js_sys::global().dyn_into().unwrap();
    let queue = Rc::new(RefCell::new(VecDeque::<QueueItem>::new()));
    let workers = Rc::new(
        (0..(global_scope.navigator().hardware_concurrency() as usize - 1)
            .max(1)
            .min(2))
            .map(|_| {
                (
                    Worker::new("terrain-worker-sub.js").unwrap(),
                    Rc::new(RefCell::new(WorkerState::NotReady)),
                )
            })
            .collect::<Vec<_>>(),
    );

    // global scope
    {
        let queue = queue.clone();
        let workers = workers.clone();

        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            let data = event.data();

            let [x, y, z] = data
                .as_string()
                .unwrap()
                .split(',')
                .flat_map(&str::parse::<i32>)
                .collect::<Vec<_>>()
                .try_into()
                .expect("invalid message given");
            let item = QueueItem::Generate((x, y, z));
            if !queue.borrow().contains(&item)
                && workers
                    .iter()
                    .all(|(_, running)| *running.borrow() != WorkerState::Ready(Some(item)))
            {
                queue.borrow_mut().push_back(item);
                trigger(&workers, &queue);
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        global_scope.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
    }

    // per worker
    for (i, (worker, running)) in workers.iter().enumerate() {
        let queue = queue.clone();
        let workers = workers.clone();
        let running = running.clone();

        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            let data = event.data();

            *running.borrow_mut() = WorkerState::Ready(None);

            wasm_bindgen_futures::spawn_local(
                QueueItem::from_message(data.as_string().unwrap()).postprocess(workers.clone(), i),
            );

            trigger(&workers, &queue);
        }) as Box<dyn FnMut(MessageEvent)>);

        worker.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
    }
}

fn trigger(
    workers: &Rc<Vec<(Worker, Rc<RefCell<WorkerState>>)>>,
    queue: &Rc<RefCell<VecDeque<QueueItem>>>,
) {
    if queue.borrow().len() == 0 {
        return;
    }
    for (worker, running) in workers.iter() {
        if *running.borrow() == WorkerState::Ready(None) {
            let item = queue.borrow_mut().pop_front().unwrap();
            *running.borrow_mut() = WorkerState::Ready(Some(item));

            worker
                .post_message(&JsValue::from_str(&item.to_message()))
                .unwrap();
            return;
        }
    }
}

impl QueueItem {
    fn from_message(message: String) -> QueueItem {
        let args = message
            .split(':')
            .flat_map(String::from_str)
            .collect::<Vec<_>>();

        match args[0].as_str() {
            "generate" => {
                let [x, y, z] = args[1]
                    .split(',')
                    .flat_map(&str::parse::<i32>)
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("Invalid message given");
                QueueItem::Generate((x, y, z))
            }
            "init" => QueueItem::Init,
            _ => panic!("Invalid message given"),
        }
    }

    fn to_message(self) -> String {
        match self {
            QueueItem::Generate((x, y, z)) => {
                format!("generate:{},{},{}", x, y, z)
            }
            QueueItem::Init => panic!("Incorrect usage"),
        }
    }

    async fn postprocess(self, workers: Rc<Vec<(Worker, Rc<RefCell<WorkerState>>)>>, i: usize) {
        match self {
            QueueItem::Generate((x, y, z)) => {
                let global_scope: DedicatedWorkerGlobalScope = js_sys::global().dyn_into().unwrap();

                // let directory: FileSystemDirectoryHandle =
                //     JsFuture::from(global_scope.navigator().storage().get_directory())
                //         .await
                //         .unwrap()
                //         .dyn_into()
                //         .unwrap();

                // let file_name = format!("0_{}_{}_{}.chunk", x, y, z);

                // let file: FileSystemFileHandle =
                //     JsFuture::from(directory.get_file_handle(&file_name))
                //         .await
                //         .unwrap()
                //         .dyn_into()
                //         .unwrap();

                // let access: FileSystemSyncAccessHandle =
                //     JsFuture::from(file.create_sync_access_handle())
                //         .await
                //         .unwrap()
                //         .dyn_into()
                //         .unwrap();

                // access
                //     .write_with_u8_array("Hello world".as_bytes())
                //     .unwrap();
                // access.flush().unwrap();
                // access.close();

                global_scope
                    .post_message(&JsValue::from_str(&format!("{},{},{}", x, y, z)))
                    .unwrap();
            }
            QueueItem::Init => {
                workers[i]
                    .0
                    .post_message(&JsValue::from_str(&format!("init:{}", i)))
                    .unwrap();
            }
        }
    }
}
