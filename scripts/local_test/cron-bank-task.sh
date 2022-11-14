#!/bin/sh
set -e

cd "$(dirname "$0")"
. ./local_init_vars.sh

BANK='{
  "create_task": {
    "task": {
      "interval": {
        "Cron": "0 * * * * *"
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
                    "amount": "1",
                    "denom": "ujunox"
                  }
                ],
                "to_address": "'$($BINARY keys show bob --address)'"
              }
            }
          }
        }
      ],
      "rules": []
    }
  }
}'

$BINARY tx wasm execute $CONTRACT_ADDRESS "$BANK" --amount 1200019ujunox --from user $TXFLAG -y
