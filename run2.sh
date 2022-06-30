#!/bin/bash

prog=`basename "$PWD"`
ps -u $USER -eo comm | grep -wq $prog && {
    kill `ps -u $USER -eo comm,pid | awk '/'$prog'/ { print $2 }'`
}

[[ -e run_me ]] && RUST_BACKTRACE=1 cargo run 

inotifywait --format %w -q -e close_write run_me

exec ./run2.sh
