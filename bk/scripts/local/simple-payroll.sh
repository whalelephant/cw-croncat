#!/bin/sh
#Usage
#sudo ./scripts/local/simple-payroll.sh juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y  -no -yes
DIR=$(pwd)
. "$DIR/scripts/local/start-in-docker.sh"


echo "${Cyan}Creating simple payroll" "${NoColor}"
# Create recurring payroll to alice and bob
SIMPLE_PAYROLL='{
  "create_task": {
    "task": {
      "interval": {
        "Block": 3
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
                    "amount": "6",
                    "denom": "'$STAKE'"
                  }
                ],
                "to_address": "'$ALICE_ADDR'"
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
                    "denom": "'$STAKE'"
                  }
                ],
                "to_address": "'$BOB_ADDR'"
              }
            }
          }
        }
      ],
      "rules": []
    }
  }
}'
echo $SIMPLE_PAYROLL
$BINARY tx wasm execute $CONTRACT_ADDRESS "$SIMPLE_PAYROLL" --amount "20000000$STAKE" --from validator $TXFLAG -y
echo "Done creating simple payroll"
