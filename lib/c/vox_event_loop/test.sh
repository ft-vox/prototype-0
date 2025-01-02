#!/bin/sh

set -e

cd "$(dirname "$0")"

cmake -DCMAKE_BUILD_TYPE=Debug -B builddir_t_std.os.thread ../std.os.thread
cmake --build builddir_t_std.os.thread --config Debug
cmake --install builddir_t_std.os.thread --prefix dependencies

cmake -DCMAKE_BUILD_TYPE=Debug -DBUILD_TESTS=ON -B builddir_self
cmake --build builddir_self --config Debug
(cd builddir_self && ctest --output-on-failure)
