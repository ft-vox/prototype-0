#!/bin/sh

set -e

cd "$(dirname "$0")"

cmake -DCMAKE_BUILD_TYPE=Release -B builddir_t_std.os.thread ../std.os.thread
cmake --build builddir_t_std.os.thread --config Release
cmake --install builddir_t_std.os.thread --prefix dependencies

cmake -DBUILD_TESTS=ON -B builddir_self
cmake --build builddir_self
(cd builddir_self && ctest --output-on-failure)
