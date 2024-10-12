#!/bin/sh

set -e

echo '[{"directory":"'"$(pwd)"'","arguments":["clang","-Iinclude","-Iexternal_headers","-x","c","-std=c99","-Wall","-Wextra","-Werror","-pedantic","-c","file.c","-o","file.o"],"file":"file.c" }]' > compile_commands.json
