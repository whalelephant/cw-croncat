#!/bin/sh
set -e

cd "$(dirname "$0")"
. ./local_init_vars.sh

BANK='{
  "create_task": {
    "task": {
      "interval": "Immediate",
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
$BINARY tx wasm execute $CONTRACT_ADDRESS "$BANK" --amount 1200018ujunox --from user $TXFLAG -y
