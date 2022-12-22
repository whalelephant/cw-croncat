set -e

SH_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SH_DIR="$(cd -P "$(dirname "${SH_PATH}")";pwd)"
. $SH_DIR/base/init-vars.sh

if [ -z "$1" ]; then
  echo "Must provide contract address"
  exit 1
elif [ -z "$2" ]; then
  echo "Must provide user address"
  exit 1
elif [ -z "$3" ]; then
  echo "Must provide rules address"
  exit 1
elif [ -z "$4" ]; then
  echo "Must provide dao address"
  exit 1
elif [ -z "$5" ]; then
  echo "Must provide agent address"
  exit 1
fi

CONTRACT="$1"
USER="$2"
RULES="$3"
DAO="$4"
AGENT="$5"

EXECUTE_MSG='{"execute":{"proposal_id":""}}'
ENCODED_EXECUTE_MSG=$(printf $EXECUTE_MSG | base64)

DAODAO='{
  "create_task": {
    "task": {
      "interval": "Immediate",
      "cw20_coins": [],
      "stop_on_fail": false,
      "actions": [
        {
          "msg": {
            "wasm": {
              "execute": {
                "contract_addr": "'$DAO'",
                "msg": "'$ENCODED_EXECUTE_MSG'",
                "funds": []
              }
            }
          },
          "gas_limit": 300000
        }
      ],
      "queries": [
        {
          "check_passed_proposals": {
            "dao_address": "'$DAO'"
          }
        }
      ],
      "transforms": [
        {
          "action_idx": 0,
          "query_idx": 0,
          "action_path": [
            {
              "key": "execute"
            },
            {
              "key": "proposal_id"
            }
          ],
          "query_response_path": []
        }
      ]
    }
  }
}'

junod tx wasm execute $CONTRACT "$DAODAO" --amount 1000000ujunox --from "$USER" $TXFLAG -y
