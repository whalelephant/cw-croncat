#!/bin/bash
set -e

NODE="--node https://rpc-juno.itastakers.com:443/"
TXFLAG="--node https://rpc-juno.itastakers.com:443/ --chain-id juno-1 --gas-prices 0.025ujuno --gas auto --gas-adjustment 1.3 --broadcast-mode block"

# Make sure OWNER has some JUNO
OWNER="$1"
CODE_ID="$2"

INIT='{"denom":"ujuno"}'
junod tx wasm instantiate $CODE_ID "$INIT" --from $OWNER --label "croncat" $TXFLAG -y --no-admin

CONTRACT=$(junod query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')
echo $CONTRACT
