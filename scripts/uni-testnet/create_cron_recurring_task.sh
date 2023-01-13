#!/bin/bash

SH_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SH_DIR="$(
    cd -P "$(dirname "${SH_PATH}")"
    pwd
)"
. $SH_DIR/base/init-vars.sh

if [ -z "$1" ]; then
    echo "Must provide contract address"
    exit 1
elif [ -z "$2" ]; then
    echo "Must provide user address"
    exit 1
else
    CONTRACT="$1"
    USER="$2"
fi

RECURRING='{
             "create_task": {
               "task": {
                 "interval": {
                   "Cron": "* */1 * * * * *"
                 },
                 "boundary": null,
                 "cw20_coins": [],
                 "stop_on_fail": false,
                 "actions": [
                   {
                     "msg": {
                       "bank": {
                         "send": {
                           "amount": [
                             {
                               "amount": "123456",
                               "denom": "ujunox"
                             }
                           ],
                           "to_address": "juno1yhqft6d2msmzpugdjtawsgdlwvgq3samrm5wrw"
                         }
                       }
                     }
                   }
                 ]
               }
             }
           }'

junod tx wasm execute $CONTRACT "$RECURRING" --amount 1600004ujunox --from $USER $TXFLAG -y

# GET_AGENT_IDS='{"get_agent_ids":{}}'
# junod query wasm contract-state smart $CONTRACT "$GET_AGENT_IDS" $NODE
