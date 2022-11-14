#!/bin/sh
set -e

cd "$(dirname "$0")"
. ./local_init_vars.sh

UNREGISTER_AGENT='{"unregister_agent":{}}'
$BINARY tx wasm execute $CONTRACT_ADDRESS "$UNREGISTER_AGENT" --from agent $TXFLAG -y