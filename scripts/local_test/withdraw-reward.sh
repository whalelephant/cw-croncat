#!/bin/sh
set -e

cd "$(dirname "$0")"
. ./local_init_vars.sh

WITHDRAW_REWARD='{"withdraw_reward":{}}'
$BINARY tx wasm execute $CONTRACT_ADDRESS "$WITHDRAW_REWARD" --from agent $TXFLAG -y