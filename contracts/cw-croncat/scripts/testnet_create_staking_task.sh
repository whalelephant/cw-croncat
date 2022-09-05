#!/bin/bash

cd "$(dirname "$0")"
. ./testnet_init_vars.sh

if [ -z "$1"]
then
    echo "Must provide contracts address"
    exit 1
elif [ -z "$2" ]
then
    echo "Must provide user address"
    exit 1
else
    CONTRACT="$1"
    USER="$2"
fi

STAKE='{"create_task":{"task":{"interval":"Immediate","boundary":null,"cw20_coins":[],"stop_on_fail":false,"actions":[{"msg":{"staking":{"delegate":{"validator":"juno14vhcdsyf83ngsrrqc92kmw8q9xakqjm0ff2dpn","amount":{"denom":"ujunox","amount":"1000000"}}}},"gas_limit":150000}],"rules":null}}}'
junod tx wasm execute $CONTRACT "$STAKE" --amount 500000ujunox --from $USER $TXFLAG -y

# GET_AGENT_IDS='{"get_agent_ids":{}}'
# junod query wasm contract-state smart $CONTRACT "$GET_AGENT_IDS" $NODE