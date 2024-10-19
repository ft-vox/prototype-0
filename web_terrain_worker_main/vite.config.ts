import { defineConfig } from "vite";
import { viteSingleFile } from "vite-plugin-singlefile";
import topLevelAwait from "vite-plugin-top-level-await";
import wasm from "vite-plugin-wasm";

export default defineConfig(({ mode }) => {
  return {
    server: { hmr: true },
    base: "./",
    plugins: [wasm(), topLevelAwait(), viteSingleFile()],
    worker: {
      plugins: () => [wasm(), topLevelAwait(), viteSingleFile()],
      format: "es",
      rollupOptions: {
        output: {
          entryFileNames: "terrain-worker-main.js",
        },
      },
    },
  };
});
