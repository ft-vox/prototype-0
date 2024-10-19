import WorkerFactory from "./worker?worker";

const worker: Worker = WorkerFactory();

window.addEventListener("click", () => {
  worker.postMessage("Hello world!");
});
