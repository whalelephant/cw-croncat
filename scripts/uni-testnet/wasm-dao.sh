set -ex

NoColor='\033[0m' # Text Reset
Red='\033[0;31m'    # Red

SH_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SH_DIR="$(cd -P "$(dirname "${SH_PATH}")";pwd)"
. $SH_DIR/base/init-vars.sh

PROPOSAL_ID=$5

if [ -z "$1" ]; then
  echo "Must provide contract address"
  exit 1
elif [ -z "$2" ]; then
  echo "Must provide user address"
  exit 1
elif [ -z "$3" ]; then
  echo "Must provide dao address"
  exit 1
elif [ -z "$4" ]; then
  echo "Must provide agent address"
  exit 1
elif [ -z "$5" ]; then
  PROPOSAL_ID=1
fi

CONTRACT="$1"
USER="$2"
DAO="$3"
AGENT="$4"

EXECUTE_MSG='{"execute":{"proposal_id":'$PROPOSAL_ID'}}'
ENCODED_EXECUTE_MSG=$(printf $EXECUTE_MSG | base64)

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
          }
        }
      ],
      "rules": []
    }
  }
}'
junod tx wasm execute $CONTRACT "$DAODAO" --amount 58433ujunox --from "$USER" $TXFLAG -y

REGISTER_AGENT='{"register_agent":{}}'
junod tx wasm execute $CONTRACT "$REGISTER_AGENT" --from $AGENT $TXFLAG -y


GET_TASKS='{"get_tasks":{}}'
junod query wasm contract-state smart $CONTRACT "$GET_TASKS" $NODE

sleep 5

GET_TASK_HASH='{
  "get_task_hash": {
    "task": {
      "owner_id": "'$(junod keys show $USER -a)'",
      "interval": "Once",
      "boundary": {
        "start": null,
        "end": null
      },
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
          }
        }
      ],
      "rules": [],
      "funds_withdrawn_recurring": [],
      "total_deposit": {
        "native": [],
        "cw20": []
      },
      "amount_for_one_task": {
        "native": [],
        "cw20": []
      }
    }
  }
}'

TASK_HASH=$(junod query wasm contract-state smart $CONTRACT "$GET_TASK_HASH" $NODE --output json | jq -r '.data')
echo $TASK_HASH

echo "${Red}AGENT balance before proxy call"
junod query bank balances "$AGENT" $NODE --denom ujunox
echo "${NoColor}"

PROXY_CALL='{"proxy_call":{}}'
junod tx wasm execute $CONTRACT "$PROXY_CALL" --from $AGENT $TXFLAG -y
junod query wasm contract-state smart $CONTRACT "$GET_TASKS" $NODE

echo "${Red}AGENT balance after proxy call"
junod query bank balances "$AGENT" $NODE --denom ujunox
echo "${NoColor}"

WITHDRAW_REWARD='{"withdraw_reward":{}}'
junod tx wasm execute $CONTRACT "$WITHDRAW_REWARD" --from $AGENT $TXFLAG -y

echo "${Red}AGENT balance after withdraw"
junod query bank balances "$AGENT" $NODE --denom ujunox
echo "${NoColor}"

# before proxy call 14396509
# after proxy call 14385203 
# after withdraw 14386002 

# pub const GAS_BASE_FEE_JUNO: u64 = 300_000;
# pub const GAS_ACTION_FEE_JUNO: u64 = 200_000;
# before proxy call 14401832
# after proxy call 14385853
# after withdraw 14439149
