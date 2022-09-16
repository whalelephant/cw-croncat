#!/bin/bash

cd "$(dirname "$0")"
. ./testnet_init_vars.sh

if [ -z "$1" ]
then
    echo "Must provide contract address"
    exit 1
elif [ -z "$2" ]
then
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
                "to_address": "juno1yhqft6d2msmzpugdjtawsgdlwvgq3samrm5wrw"
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
                    "amount": "1",
                    "denom": "ujunox"
                  }
                ],
                "to_address": "juno15w7hw4klzl9j2hk4vq7r3vuhz53h3mlzug9q6s"
              }
            }
          }
        }
      ],
      "rules": []
    }
  }
}'
junod tx wasm execute $CONTRACT "$RECURRING" --amount 1600004ujunox --from $USER $TXFLAG -y

# GET_AGENT_IDS='{"get_agent_ids":{}}'
# junod query wasm contract-state smart $CONTRACT "$GET_AGENT_IDS" $NODE
