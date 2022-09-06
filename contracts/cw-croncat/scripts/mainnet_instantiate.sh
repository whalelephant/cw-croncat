#!/bin/bash
set -e

NODE="--node https://rpc-juno.itastakers.com:443/"
TXFLAG="--node https://rpc-juno.itastakers.com:443/ --chain-id juno-1 --gas-prices 0.025ujuno --gas auto --gas-adjustment 1.3 --broadcast-mode block"

# Make sure OWNER has some JUNO
if [ -z "$1" ]
then
    echo "Must provide owner address"
    exit 1
elif [ -z "$2" ]
then
    echo "Must provide owner code id"
    exit 1
else
    OWNER="$1"
    CODE_ID="$2"
fi

INIT='{"denom":"ujuno"}'
junod tx wasm instantiate $CODE_ID "$INIT" --from $OWNER --label "croncat" $TXFLAG -y --no-admin

CONTRACT=$(junod query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')
echo $CONTRACT
