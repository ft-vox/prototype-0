#!/bin/sh

set -e

wasm-pack build core
(cd web && npm i && npx vite dev)
