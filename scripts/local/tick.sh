#!/bin/bash
set -ex

CHAIN_ID="testing"
RPC="http://localhost:26657/"
BINARY="docker exec -i juno_node_1 junod"
DIR=$(pwd)
JUNO_DIR="$HOME/juno"
DIR_NAME=$(basename "$PWD")
IMAGE_NAME="juno_node_1"
DIR_NAME_SNAKE=$(echo $DIR_NAME | tr '-' '_')
WASM="artifacts/$DIR_NAME_SNAKE.wasm"
STAKE_TOKEN=ujunox
STAKE=${STAKE_TOKEN:-ustake}
TXFLAG="--gas-prices 0.075$STAKE --gas auto --gas-adjustment 1.2 -y -b block --chain-id $CHAIN_ID --node $RPC"

CONTRACT_ADDRESS="$1"
ALICE="owner"

UPDATE_SETTINGS='{
  "update_settings": {
    "min_tasks_per_agent": 1,
    "agents_eject_threshold": 2,
    "slot_granularity": 2
  }
}'
$BINARY tx wasm execute $CONTRACT_ADDRESS "$UPDATE_SETTINGS" --from validator $TXFLAG -y

TICK='{"tick":{}}'
ENCODED_TICK=$(echo $TICK | base64)
TICK_TASK='{
  "create_task": {
    "task": {
      "interval": "Immediate",
      "boundary": null,
      "cw20_coins": [],
      "stop_on_fail": false,
      "actions": [
        {
          "msg": {
            "wasm": {
              "execute": {
                "contract_addr": "'$CONTRACT_ADDRESS'",
                "msg": "'$ENCODED_TICK'",
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

$BINARY tx wasm execute $CONTRACT_ADDRESS "$TICK_TASK" --amount "20000000$STAKE" --from validator $TXFLAG -y

REGISTER_AGENT='{"register_agent":{}}'
$BINARY tx wasm execute $CONTRACT_ADDRESS "$REGISTER_AGENT" --from validator $TXFLAG -y
$BINARY tx wasm execute $CONTRACT_ADDRESS "$REGISTER_AGENT" --from $ALICE $TXFLAG -y

# Make agent active
CHECK_IN_AGENT='{"check_in_agent":{}}'
$BINARY tx wasm execute $CONTRACT_ADDRESS "$CHECK_IN_AGENT" --from $ALICE $TXFLAG -y

PROXY_CALL='{"proxy_call":{}}'
$BINARY tx wasm execute $CONTRACT_ADDRESS "$PROXY_CALL" --from validator $TXFLAG -y

sleep 20

PROXY_CALL='{"proxy_call":{}}'
$BINARY tx wasm execute $CONTRACT_ADDRESS "$PROXY_CALL" --from validator $TXFLAG -y
