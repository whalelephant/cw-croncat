. ./scripts/uni-testnet/base/init-vars.sh
if [ -z "$1" ]; then
  echo "Must provide contract address"
  exit 1
elif [ -z "$2" ]; then
  echo "Must provide user address"
  exit 1
elif [ -z "$3" ]; then
  echo "Must provide dao address"
  exit 1
fi

CONTRACT="$1"
USR="$2"
DAO="$3"


MSG='{"get_balance":{"address":"'$USR'","denom":"ujunox"}}'

DAODAO='{
  "create_task": {
    "task": {
      "interval": "Once",
      "boundary": null,
      "stop_on_fail": false,
      "actions": [
        {
          "msg": {
            "bank": {
              "send": {
                "to_address": "$USER_ADDRESS",
                "amount": [
                  {
                    "denom": "ujunox",
                    "amount": "1"
                  }
                ]
              }
            }
          },
          "gas_limit": null
        }
      ],
      "rules": [
        {
          "check_proposal_status": {
            "dao_address": "'$DAO'",
            "proposal_id": 1,
            "status": "passed"
          }
        }
      ],
      "cw20_coins": []
    }
  }
}'

junod tx wasm execute $CONTRACT "$DAODAO" --amount 1000000ujunox --from signer $TXFLAG -y
