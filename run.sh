#!/bin/bash

clear

cargo fmt
cargo clippy &&
RUST_BACKTRACE=1 cargo build && {
    killall rspread
    touch run_me
 } || 
    echo --------------------------------------------------------------------------------

inotifywait -q -e close_write src ../rzdb/src Cargo.toml run.sh run2.sh

exec ./run.sh