#!/bin/bash

cargo fmt
cargo clippy &&
RUST_BACKTRACE=1 cargo build && touch run_me || 
    echo --------------------------------------------------------------------------------

inotifywait -q -e close_write src Cargo.toml run.sh
clear

exec ./run.sh