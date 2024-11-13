#!/bin/sh

set -e

do_work() {
    DIR="$1"

    for DIR in "$DIR"/*; do
        if [ -d "$DIR" ]; then
            if [ -f "$DIR/print_compile_commands.sh" ]; then
                (cd "$DIR" && sh print_compile_commands.sh)
            else
                do_work "$DIR"
            fi
        fi
    done
}

(echo '[' && do_work lib && echo ']') > compile_commands.json
