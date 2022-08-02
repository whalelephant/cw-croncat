#!/bin/bash
set -e

sh build.sh
# In case of M1 MacBook use rust-optimizer-arm64 instead of rust-optimizer
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.6

TXFLAG="--node https://rpc-juno.itastakers.com:443/ --chain-id juno-1 --gas-prices 0.025ujuno --gas auto --gas-adjustment 1.3 --broadcast-mode block"

# Make sure OWNER has enough JUNO (about 2 JUNO)
OWNER="$1"

# In case of M1 MacBook replace cw_croncat.wasm with cw_croncat-aarch64.wasm 
RES=$(junod tx wasm store artifacts/cw_croncat.wasm --from $OWNER $TXFLAG -y --output json -b block)
CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value')
echo $CODE_ID  
