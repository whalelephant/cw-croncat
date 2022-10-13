#!/bin/bash
set -e

export RUSTFLAGS='-C link-arg=-s'

cargo fmt --all
cargo clippy -- -D warnings
cargo build --release --lib --target wasm32-unknown-unknown