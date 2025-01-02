#!/bin/sh

set -e

cmake -DBUILD_TESTS=ON -B builddir
cmake --build builddir --config Debug
(cd builddir && ctest -C Debug --output-on-failure)
