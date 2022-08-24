#!/bin/bash
set -ex

# If call from scripts/
# cd "$(dirname "$0")/.."

# Deploy cw_rules contract
. ./scripts/cw-rules_deploy_testnet.sh
RULES_CONTRACT=$CONTRACT

# Create wallets and make sure they have some JUNOX
OWNER=owner$RANDOM
AGENT=agent$RANDOM
USER=user$RANDOM

junod keys add $OWNER
junod keys add $AGENT
junod keys add $USER

# Make sure they have 10 JUNOX each
JSON=$(jq -n --arg addr $(junod keys show -a $OWNER) '{ denom:"ujunox","address":$addr}') && \
  curl -X POST --header "Content-Type: application/json" --data "$JSON" https://faucet.uni.juno.deuslabs.fi/credit && echo
JSON=$(jq -n --arg addr $(junod keys show -a $AGENT) '{ denom:"ujunox","address":$addr}') && \
  curl -X POST --header "Content-Type: application/json" --data "$JSON" https://faucet.uni.juno.deuslabs.fi/credit && echo
JSON=$(jq -n --arg addr $(junod keys show -a $USER) '{ denom:"ujunox","address":$addr}') && \
  curl -X POST --header "Content-Type: application/json" --data "$JSON" https://faucet.uni.juno.deuslabs.fi/credit && echo

echo "Created $OWNER with 10 JUNOX balance"
echo "Created $AGENT with 10 JUNOX balance"
echo "Created $USER with 10 JUNOX balance"

# In case of M1 MacBook replace cw_croncat.wasm with cw_croncat-aarch64.wasm 
RES=$(junod tx wasm store artifacts/cw_croncat.wasm --from $OWNER $TXFLAG -y --output json -b block)
CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value')

# Instantiate
INIT='{"denom":"ujunox"}'
junod tx wasm instantiate $CODE_ID "$INIT" --from $OWNER --label "croncat" $TXFLAG -y --no-admin
CONTRACT=$(junod query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')

# Now we can register an agent, create tasks and execute a task
# Register an agent
REGISTER_AGENT='{"register_agent":{}}'
junod tx wasm execute $CONTRACT "$REGISTER_AGENT" --from $AGENT $TXFLAG -y

MSG='{"get_balance":{"address":"'$(junod keys show $USER -a)'","denom":"ujunox"}}'
ENCODED_MSG=$(echo $MSG | base64)

# Create a task
STAKE='{
  "create_task": {
    "task": {
      "interval": "Once",
      "boundary": null,
      "cw20_coins": [],
      "stop_on_fail": false,
      "actions": [
        {
          "msg": {
            "staking": {
              "delegate": {
                "validator": "juno14vhcdsyf83ngsrrqc92kmw8q9xakqjm0ff2dpn",
                "amount": {
                  "denom": "ujunox",
                  "amount": "1000000"
                }
              }
            }
          },
          "gas_limit": 150000
        }
      ],
      "rules": [
        {
          "contract_addr": "'$RULES_CONTRACT'",
          "msg": "'$ENCODED_MSG'"
        }
      ]
    }
  }
}'
junod tx wasm execute $CONTRACT "$STAKE" --amount 1000000ujunox --from $USER $TXFLAG -y

# See tasks with rules
GET_TASKS_WITH_RULES='{"get_tasks_with_rules":{}}'
TASKS_WITH_RULES=$(junod query wasm contract-state smart $CONTRACT "$GET_TASKS_WITH_RULES" $NODE --output json)
junod query wasm contract-state smart $CONTRACT "$GET_TASKS_WITH_RULES" $NODE

# proxy_call
sleep 10      # is needed to make sure this call in the next block 
TASK_HASH=$(echo $TASKS_WITH_RULES | jq -r '.data[0].task_hash')
PROXY_CALL='{"proxy_call":{"task_hash":"'$TASK_HASH'"}}'
junod tx wasm execute $CONTRACT "$PROXY_CALL" --from $AGENT $TXFLAG -y

# There shouldn't be any tasks left
junod query wasm contract-state smart $CONTRACT "$GET_TASKS_WITH_RULES" $NODE

echo "CONTRACT CODEID - $CODE_ID"
echo "CONTRACT $CONTRACT"
echo "OWNER $OWNER"
echo "AGENT $AGENT"
echo "USER $USER"

