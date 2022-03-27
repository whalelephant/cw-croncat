#!/bin/bash
set -e

export RUSTFLAGS='-C link-arg=-s'

cargo unit-test --locked
cargo wasm --locked