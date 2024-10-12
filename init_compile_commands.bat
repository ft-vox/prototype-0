@echo off

setlocal EnableDelayedExpansion

set "directory=%cd%"
set "escapedDirectory=!directory:\=\\!"

(echo [^{"directory":"!escapedDirectory!","arguments":["clangd","-Iinclude","-Iexternal_headers","-x","c","-std=c99","-Wall","-Wextra","-Werror","-pedantic","-c","file.c","-o","file.o"],"file":"file.c"^}]) > compile_commands.json
