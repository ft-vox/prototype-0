#!/bin/sh

set -e

cd "$(dirname "$0")"

bindgen ../../c/mod/include/mod.h --allowlist-function 'get_any_unresolved_map_dependency' --allowlist-function 'get_all_unresolved_map_dependencies' --allowlist-type 'Mod' -o src/mod_bindings.rs -- -I../../c/TMap/include
