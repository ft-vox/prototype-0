#!/bin/sh

set -e

echo '{"directory":"'"$(pwd)"'","arguments":["clang","-Iinclude","-I../std.os.thread/include","-x","c","-std=c99","-Wall","-Wextra","-Werror","-pedantic","-c","file.c","-o","file.o"],"file":"file.c" },'
