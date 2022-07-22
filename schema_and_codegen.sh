#!/bin/bash
set -e

cargo run --example schema
# if it fails install:
# npm install -g cosmwasm-typescript-gen
cosmwasm-typescript-gen generate --schema ./packages/cw-croncat-core/schema --out ./types/contract --name cw-croncat