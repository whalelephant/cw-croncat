#!/bin/sh
set -e

# cd "$(dirname "$0")"
# . ./start-in-docker.sh -c

CHAIN_ID="testing"
RPC="http://localhost:26657/"
STAKE_TOKEN=ujunox
STAKE=${STAKE_TOKEN:-ustake}
TXFLAG="--gas-prices 0.075$STAKE --gas auto --gas-adjustment 1.2 -y -b block --chain-id $CHAIN_ID --node $RPC"
BINARY="docker exec -i juno_node_1 junod"

Green='\033[0;32m'  # Green
NoColor='\033[0m'   # Text Reset

CONTRACT_ADDRESS=$1

BANK='{
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
                    "amount": "2",
                    "denom": "ujunox"
                  }
                ],
                "to_address": "'$($BINARY keys show alice --address)'"
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
$BINARY tx wasm execute $CONTRACT_ADDRESS "$BANK" --amount 500000"$STAKE_TOKEN" --from user $TXFLAG -y

REGISTER_AGENT='{"register_agent":{}}'
$BINARY tx wasm execute $CONTRACT_ADDRESS "$REGISTER_AGENT" --from agent $TXFLAG -y

AGENT_BALANCE_BEFORE=$($BINARY q bank balances $($BINARY keys show agent --address < /dev/null) < /dev/null)
USER_BALANCE_BEFORE=$($BINARY q bank balances $($BINARY keys show user --address < /dev/null) < /dev/null)

PROXY_CALL='{"proxy_call":{}}'
PROXY_CALL_COUNT=0
while [ "$PROXY_CALL_COUNT" -lt 6 ]; do
  $BINARY tx wasm execute $CONTRACT_ADDRESS "$PROXY_CALL" --from agent $TXFLAG -y < /dev/null

  ALICE_BALANCE=$($BINARY q bank balances $($BINARY keys show alice --address < /dev/null) < /dev/null)
  echo "${Green}Alice Balance :" $ALICE_BALANCE "${NoColor}"
  BOB_BALANCE=$($BINARY q bank balances $($BINARY keys show bob --address < /dev/null) < /dev/null)
  echo "${Green}Bob Balance :" $BOB_BALANCE "${NoColor}"
  
  PROXY_CALL_COUNT="$((PROXY_CALL_COUNT+1))"
  echo "\033[0;35m Number of proxy calls: $PROXY_CALL_COUNT "${NoColor}""

  GET_STATE='{"get_state": {}}'
  $BINARY query wasm contract-state smart $CONTRACT_ADDRESS "$GET_STATE" --node "$RPC" < /dev/null

  sleep 6
done

USER_BALANCE_AFTER=$($BINARY q bank balances $($BINARY keys show user --address < /dev/null) < /dev/null)

# 101641 183310 264979 346648 428317 509986 591655 673324 754993 836662 918331
# 81669 per one execution
# 19972 should be returned to the user

AGENT_BALANCE_AFTER=$($BINARY q bank balances $($BINARY keys show agent --address < /dev/null) < /dev/null)

WITHDRAW_REWARD='{"withdraw_reward":{}}'
$BINARY tx wasm execute $CONTRACT_ADDRESS "$WITHDRAW_REWARD" --from agent $TXFLAG -y

AGENT_BALANCE=$($BINARY q bank balances $($BINARY keys show agent --address < /dev/null) < /dev/null)

echo "${Green}Agent Balance before proxy calls:" $AGENT_BALANCE_BEFORE
echo "Agent Balance after proxy calls:" $AGENT_BALANCE_AFTER
echo "Agent Balance after withdraw:" $AGENT_BALANCE

echo "User Balance before proxy calls:" $USER_BALANCE_BEFORE
echo "User Balance after proxy calls:" $USER_BALANCE_AFTER"${NoColor}"
