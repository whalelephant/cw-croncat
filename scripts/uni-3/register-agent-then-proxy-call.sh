#!/bin/bash
set -e

cd "$(dirname "$0")"
. ./init-vars.sh

if [ -z "$1" ]
then
    echo "Must provide contract address"
    exit 1
elif [ -z "$2" ]
then
    echo "Must provide address of the new agent"
    exit 1
else
    CONTRACT="$1"
    AGENT="$2"
fi

REGISTER_AGENT='{"register_agent":{}}'
junod tx wasm execute $CONTRACT "$REGISTER_AGENT" --from $AGENT $TXFLAG -y

# Make agent active
CHECK_IN_AGENT='{"check_in_agent":{}}'
junod tx wasm execute $CONTRACT "$CHECK_IN_AGENT" --from $AGENT $TXFLAG -y

PROXY_CALL='{"proxy_call":{}}'
junod tx wasm execute $CONTRACT "$PROXY_CALL" --from $AGENT $TXFLAG -y

echo "AGENT - " $AGENT
echo "CONTRACT - " $CONTRACT
