#!/bin/bash

old_timestamp=${1:-""}

prog=`basename "$PWD"`
ps -u $USER -eo comm | grep -wq $prog && {
    kill `ps -u $USER -eo comm,pid | awk '/'$prog'/ { print $2 }'`
}

[[ -e run_me ]] && RUST_BACKTRACE=1 cargo run
if [[ ! -z "$old_timestamp" ]]; then
    new_timestamp=`date +%s`
    diff=$((new_timestamp - old_timestamp))
    if [[ $diff -lt 2 ]]; then
        sleep_time=30
        echo "Compile cycle is too fast, sleeping for $sleep_time seconds"
        sleep $sleep_time
    fi
fi

exec ./run2.sh `date +%s`