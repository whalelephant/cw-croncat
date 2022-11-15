source ~/.profile

SH_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SH_DIR="$(cd -P "$(dirname "${SH_PATH}")";pwd)"
. $SH_DIR/base/init-vars.sh

if [ -z "$1" ]; then
  echo "Must provide contract address"
  exit 1
elif [ -z "$2" ]; then
  echo "Must provide user address"
  exit 1
fi

CONTRACT="$1"
USR="$2"
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
                "to_address": "'$USR'",
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
      "rules": [],
      "cw20_coins": []
    }
  }
}';

junod tx wasm execute $CONTRACT "$DAODAO" --amount 1000000ujunox --from signer $TXFLAG -y
