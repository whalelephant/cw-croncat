#!/bin/bash
set -e

cd "$(dirname "$0")"/../../../

# OWNER must have enough JUNO (about 2 JUNO)
if [ -z "$1" ]
then
    echo "Must provide owner address"
    exit 1
else 
    OWNER=$1
fi

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  --platform linux/amd64 \
  cosmwasm/workspace-optimizer:0.12.6

TXFLAG="--node https://rpc-juno.itastakers.com:443/ --chain-id juno-1 --gas-prices 0.025ujuno --gas auto --gas-adjustment 1.3 --broadcast-mode block"

RES=$(junod tx wasm store artifacts/cw_croncat-aarch64.wasm --from $OWNER $TXFLAG -y --output json -b block)
CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value')
echo $CODE_ID  
