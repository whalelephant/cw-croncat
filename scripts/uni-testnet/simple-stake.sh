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
      "interval": "Immediate",
      "boundary": null,
      "cw20_coins": [],
      "stop_on_fail": false,
      "actions": [
        {
          "msg": {
            "staking": {
              "delegate": {
                "validator": "junovaloper18wgy6hy6yv3fvevl5pyfn7cvzx3t5use2vssnf",
                "amount": {
                  "denom": "ujunox",
                  "amount": "1"
                }
              }
            }
          }
        },
        {
          "msg": {
            "staking": {
              "delegate": {
                "validator": "junovaloper1fef7p87vdwn0mvmlh7gpeaq5jn3znkm0dhkw79",
                "amount": {
                  "denom": "ujunox",
                  "amount": "1"
                }
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
