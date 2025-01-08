#!/bin/sh

set -e

cd "$(dirname "$0")"

bindgen ../../c/mod/include/mod_types.h --allowlist-type 'Mod' -o src/mod_bindings.rs -- -I../../c/TMap/include
