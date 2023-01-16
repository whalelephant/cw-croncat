#!/bin/bash
set -e

SH_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SH_DIR="$(cd -P "$(dirname "${SH_PATH}")";pwd)"
. $SH_DIR/base/init-vars.sh

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

TASK='{
  "create_task": {
    "task": {
      "interval": "Once",
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
                "to_address": "juno1njf5qv8ryfl07qgu5hqy8ywcvzwyrt4kzqp07d"
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
                "to_address": "juno1pd43m659naajmn2chkt6tna0uud2ywyp5dm4h3"
              }
            }
          }
        }
      ],
      "rules": []
    }
  }
}'
junod tx wasm execute $CONTRACT "$TASK" --amount 1000000ujunox --from $USER $TXFLAG -y

