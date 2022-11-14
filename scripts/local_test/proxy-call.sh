#!/bin/sh
set -e

cd "$(dirname "$0")"
. ./local_init_vars.sh

PROXY_CALL='{"proxy_call":{}}'
$BINARY tx wasm execute $CONTRACT_ADDRESS "$PROXY_CALL" --from agent $TXFLAG -y
