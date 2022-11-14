#!/bin/sh
set -e

cd "$(dirname "$0")"
. ./local_init_vars.sh

BANK_IMMEDIATE='{
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
$BINARY tx wasm execute $CONTRACT_ADDRESS "$BANK_IMMEDIATE" --amount 1200018ujunox --from validator $TXFLAG -y

sleep 2

STAKE_ONCE='{
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

$BINARY tx wasm execute $CONTRACT_ADDRESS "$STAKE_ONCE" --amount 150206ujunox --from validator $TXFLAG -y

sleep 2

BANK_CRON='{
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

$BINARY tx wasm execute $CONTRACT_ADDRESS "$BANK_CRON" --amount 1200019ujunox --from validator $TXFLAG -y

BANK_BLOCK='{
  "create_task": {
    "task": {
      "interval": {
        "Block": 15
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
        },
        {
          "msg": {
            "bank": {
              "send": {
                "amount": [
                  {
                    "amount": "2",
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

$BINARY tx wasm execute $CONTRACT_ADDRESS "$BANK_BLOCK" --amount 1600017ujunox --from validator $TXFLAG -y # execute the task twice, 1ujunox is returned

