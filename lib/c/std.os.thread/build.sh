#!/bin/sh

set -e

cmake -DCMAKE_BUILD_TYPE=Release -B builddir
cmake --build builddir --config Release
