#!/bin/sh

set -e

wasm-pack build lib
(cd web && npm i && npx vite dev)
