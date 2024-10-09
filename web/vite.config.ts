import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";

export default defineConfig(({ mode }) => {
  return {
    server: { hmr: true },
    plugins: [
      react({
        include: ["**/*.tsx", "**/*.ts"],
      }),
      wasm(),
    ],
  };
});
