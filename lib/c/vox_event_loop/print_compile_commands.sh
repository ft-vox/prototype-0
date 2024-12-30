#!/bin/sh

set -e

echo '{"directory":"'"$(pwd)"'","arguments":["clang","-Iinclude","-x","c","-std=c99","-Wall","-Wextra","-Werror","-pedantic","-c","file.c","-o","file.o"],"file":"file.c" },'
