@echo off

setlocal EnableDelayedExpansion

set "directory=%cd%"
set "escapedDirectory=!directory:\=\\!"

echo ^{"directory":"!escapedDirectory!","arguments":["clangd","-Iinclude","-I../std.os.thread/include","-x","c","-std=c99","-Wall","-Wextra","-Werror","-pedantic","-c","file.c","-o","file.o"],"file":"file.c"^},
