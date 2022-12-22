#!/bin/bash
set -e

SH_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SH_DIR="$(cd -P "$(dirname "${SH_PATH}")";pwd)"
. $SH_DIR/base/init-vars.sh

if [ -z "$1" ]
then
    echo "Must provide contract address"
    exit 1
elif [ -z "$2" ]
then
    echo "Must provide address of the agent"
    exit 1
elif [ -z "$3" ]
then
    echo "Must provide the task hash"
    exit 1
else
    CONTRACT="$1"
    AGENT="$2"
    HASH="$3"
fi


PROXY_CALL='{"proxy_call":{"task_hash": "'$HASH'"}}'
junod tx wasm execute $CONTRACT "$PROXY_CALL" --from $AGENT $TXFLAG -y
