#!/bin/sh
set -e

cd "$(dirname "$0")"
. ./local_init_vars.sh

STAKE='{
  "create_task": {
    "task": {
      "interval": "Once",
      "boundary": null,
      "cw20_coins": [],
      "stop_on_fail": false,
      "actions": [
        {
          "msg": {
            "staking": {
              "delegate": {
                "validator": "'$($BINARY keys show validator --address)'",
                "amount": {
                  "denom": "ujunox",
                  "amount": "200"
                }
              }
            }
          },
          "gas_limit": 150000
        }
      ],
      "rules": null
    }
  }
}'
$BINARY tx wasm execute $CONTRACT_ADDRESS "$STAKE" --amount 150205ujunox --from user $TXFLAG -y
