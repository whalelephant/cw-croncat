#!/bin/sh
source ~/.profile
SH_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SH_DIR="$(cd -P "$(dirname "${SH_PATH}")";pwd)"
SC_PATH="$(cd -P "$(dirname "${SH_PATH}")/../..";pwd)"
SCRIPTS_PATH="$(cd -P "$(dirname "${SH_PATH}")/..";pwd)"

SH_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SH_DIR="$(cd -P "$(dirname "${SH_PATH}")";pwd)"
. $SH_DIR/base/init-vars.sh

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

echo "Removing existing agent..."
UNREGISTER_AGENT='{"unregister_agent":{}}'
junod tx wasm execute $CONTRACT "$UNREGISTER_AGENT" --from $AGENT $TXFLAG -y
sleep 5

echo "Registering agent..."
REGISTER_AGENT='{"register_agent":{}}'
junod tx wasm execute $CONTRACT "$REGISTER_AGENT" --from $AGENT $TXFLAG -y
sleep 5

# Make agent active
echo "Make agent active..."
CHECK_IN_AGENT='{"check_in_agent":{}}'
junod tx wasm execute $CONTRACT "$CHECK_IN_AGENT" --from $AGENT $TXFLAG -y
sleep 2
echo "Making proxy_call..."

PROXY_CALL='{"proxy_call":{}}'
junod tx wasm execute $CONTRACT "$PROXY_CALL" --from $AGENT $TXFLAG -y

echo "AGENT - " $AGENT
echo "CONTRACT - " $CONTRACT
