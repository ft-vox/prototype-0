#!/bin/sh

set -e

wasm-pack build wasm_terrain_worker_main
(cd web_terrain_worker_main && npm i && npx vite build)
cp web_terrain_worker_main/dist/terrain-worker-main.js web/public

wasm-pack build wasm_terrain_worker_sub
(cd web_terrain_worker_sub && npm i && npx vite build)
cp web_terrain_worker_sub/dist/terrain-worker-sub.js web/public

wasm-pack build main
(cd web && npm i && npx vite dev)
