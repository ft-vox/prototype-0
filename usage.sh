#!/bin/sh

set -e

cmake -B builddir
cmake --build builddir
