#!/bin/sh
set -e

cd "$(dirname "$0")"
. ./local_init_vars.sh

REGISTER_AGENT='{
  "register_agent": {
    "payable_account_id": "'$($BINARY keys show alice -a)'"
  }
}'
$BINARY tx wasm execute $CONTRACT_ADDRESS "$REGISTER_AGENT" --from agent $TXFLAG -y