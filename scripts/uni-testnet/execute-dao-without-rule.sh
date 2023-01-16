set -ex

SH_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SH_DIR="$(cd -P "$(dirname "${SH_PATH}")";pwd)"
. $SH_DIR/base/init-vars.sh

PROPOSAL_ID=$6

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
elif [ -z "$6" ]; then
  PROPOSAL_ID=1
fi

CONTRACT="$1"
USER="$2"
RULES="$3"
DAO="$4"
AGENT="$5"

EXECUTE_MSG='{"execute":{"proposal_id":''}}'
ENCODED_EXECUTE_MSG=$(printf $EXECUTE_MSG | base64)

# ENCODED_EXECUTE_MSG=$(echo -n '{"execute":{"proposal_id":''}}' | base64)

DAODAO='{
  "create_task": {
    "task": {
      "interval": "Once",
      "boundary": null,
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
          "gas_limit": 200000
        }
      ],
      "queries": [
        {
          "check_owner_of_nft": {

          }
        },
      ],
      "transforms": [
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
          "query_response_path": [
            {
              "index": 0
            }
          ]
      ]
    }
  }
}'
# BALANCE='{
#   "create_task": {
#     "task": {
#       "interval": "Once",
#       "boundary": null,
#       "cw20_coins": [],
#       "stop_on_fail": false,
#       "actions": [],
#       "rules": [
#         {
#           "has_balance_gte": {
#             "address": "juno1neklhchn779hl5ypr6wr36yc0dwgdmwjsvu2ns",
#             "required_balance": {
#               "native": [
#                 {
#                   "denom": "ujunox",
#                   "amount": "1"
#                 }
#               ]
#             }
#           }
#         }
#       ]
#     }
#   }
# }'
junod tx wasm execute $CONTRACT "$DAODAO" --amount 55000ujunox --from "$USER" $TXFLAG -y

GET_TASKS='{"get_tasks":{}}'
junod query wasm contract-state smart $CONTRACT "$GET_TASKS" $NODE

# sleep 5

# GET_TASK_HASH='{
#   "get_task_hash": {
#     "task": {
#       "owner_id": "'$(junod keys show $USER -a)'",
#       "interval": "Once",
#       "boundary": {
#         "start": null,
#         "end": null
#       },
#       "stop_on_fail": false,
#       "actions": [
#         {
#           "msg": {
#             "wasm": {
#               "execute": {
#                 "contract_addr": "'$DAO'",
#                 "msg": "'$ENCODED_EXECUTE_MSG'",
#                 "funds": []
#               }
#             }
#           },
#           "gas_limit": 400000
#         }
#       ],
#       "rules": [],
#       "total_deposit": {
#         "native": [],
#         "cw20": []
#       },
#       "amount_for_one_task": {
#         "native": [],
#         "cw20": []
#       }
#     }
#   }
# }'

# TASK_HASH=$(junod query wasm contract-state smart $CONTRACT "$GET_TASK_HASH" $NODE --output json | jq -r '.data')
# echo $TASK_HASH

# ALICE_BALANCE_BEFORE=$(junod q bank balances $(junod keys show alice --address)  $NODE)

# PROXY_CALL='{"proxy_call":{}}'
# junod tx wasm execute $CONTRACT "$PROXY_CALL" --from $AGENT $TXFLAG -y
# junod query wasm contract-state smart $CONTRACT "$GET_TASKS" $NODE

# ALICE_BALANCE_AFTER=$(junod q bank balances $(junod keys show alice --address)  $NODE)

# echo "${Green}Alice Balance before:" $ALICE_BALANCE_BEFORE "${NoColor}"
# echo "${Green}Alice Balance after:" $ALICE_BALANCE_AFTER "${NoColor}"
