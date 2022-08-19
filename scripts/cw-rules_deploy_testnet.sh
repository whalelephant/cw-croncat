#!/bin/bash
set -e

# In case of M1 MacBook use rust-optimizer-arm64 instead of rust-optimizer
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.6

NODE="--node https://rpc.uni.juno.deuslabs.fi:443"
TXFLAG="--node https://rpc.uni.juno.deuslabs.fi:443 --chain-id uni-3 --gas-prices 0.025ujunox --gas auto --gas-adjustment 1.3 --broadcast-mode block"


if [ -z "$1"]
then
    OWNER=cw-test-rules-owner
    junod keys show cw-test-rules-owner || junod keys add $OWNER
    JSON=$(jq -n --arg addr $(junod keys show -a $OWNER) '{ denom:"ujunox","address":$addr}') && \
  curl -X POST --header "Content-Type: application/json" --data "$JSON" https://faucet.uni.juno.deuslabs.fi/credit && echo
else 
    OWNER=$1
fi

# In case of M1 MacBook replace cw_croncat.wasm with cw_croncat-aarch64.wasm
RES=$(junod tx wasm store artifacts/cw_rules.wasm --from $OWNER $TXFLAG -y --output json -b block)
CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value')

INIT='{}'
junod tx wasm instantiate $CODE_ID "$INIT" --from $OWNER --label "croncat_rules" $TXFLAG -y --no-admin
CONTRACT=$(junod query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')

echo -e "\n\n"
echo -e "CODE_ID=$CODE_ID"
echo -e "CONTRACT=$CONTRACT"