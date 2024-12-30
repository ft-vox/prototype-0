@echo off
setlocal enabledelayedexpansion

set ERRORLEVEL=0
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%

:do_work
(
    echo [
    for /d %%D in (lib\c\*) do (
        if exist "%%D" (
            if exist "%%D\print_compile_commands.bat" (
                pushd "%%D"
                call print_compile_commands.bat
                popd
            )
        )
    )
    echo ]
) > compile_commands.json
exit /b
