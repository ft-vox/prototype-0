#!/bin/sh

set -e

wasm-pack build main
(cd web && npm i && npx vite dev)
