#!/bin/sh

set -e

do_work() {
    echo '['
    for DIR in lib/c/*; do
        if [ -d "$DIR" ]; then
            if [ -f "$DIR/print_compile_commands.sh" ]; then
                (cd "$DIR" && sh print_compile_commands.sh)
            fi
        fi
    done
    echo ']'
}

do_work > compile_commands.json
