#!/bin/sh

set -e

cmake -DBUILD_TESTS=ON -B builddir
cmake --build builddir
(cd builddir && ctest --output-on-failure)
