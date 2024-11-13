#!/bin/sh

set -e

cd "$(dirname "$0")"

bindgen ../../c/TMap/src/internal.h -o src/tmap_bindings.rs
