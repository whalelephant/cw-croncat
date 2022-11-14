#!/bin/sh
set -e

cd "$(dirname "$0")"
. ./local_init_vars.sh

GET_TASKS='{"get_tasks":{}}'
$BINARY query wasm contract-state smart $CONTRACT_ADDRESS "$GET_TASKS" $NODE

GET_STATE='{
  "get_state": {}
}'
$BINARY query wasm contract-state smart $CONTRACT_ADDRESS "$GET_STATE" $NODE
