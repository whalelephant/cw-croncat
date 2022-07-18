# Build the contract
Building:
```bash
cargo wasm
```
Optimizing the binary size:
```bash
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.6
```
> In case of M1 MacBook use `rust-optimizer-arm64` instead of `rust-optimizer`

# Setup the variables
Set up these variables, so that you don't have to type in node, chain id and gas-price details every time you execute commands.
```bash
NODE="--node https://rpc.uni.juno.deuslabs.fi:443"
TXFLAG="--node https://rpc.uni.juno.deuslabs.fi:443 --chain-id uni-3 --gas-prices 0.025ujunox --gas auto --gas-adjustment 1.3 --broadcast-mode block"
```
Specify your addresses, which will be responsible for owner of the contract, agent and the user, who creates tasks:
```bash
OWNER=your-owner-address
USER=creator-of-the-tasks-address
AGENT=your-agent-address
```
Note, that `OWNER`'s balance must be enough for storing a wasm file.
`USER` will also spend some tokens creating tasks.

# Store the code
Store the code to the uni-3 testnet:
```bash
RES=$(junod tx wasm store artifacts/cw_croncat.wasm --from $OWNER $TXFLAG -y --output json -b block)
CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value')
```
> In case of M1 MacBook replace `cw_croncat.wasm` with `cw_croncat-aarch64.wasm` 
# Instantiate the contract
```bash
INIT='{"denom":"ujunox"}'
junod tx wasm instantiate $CODE_ID "$INIT" --from $OWNER --label "croncat" $TXFLAG -y --no-admin
CONTRACT=$(junod query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')
```
# Interacting with croncat

## Tasks
Here `USER` creates three tasks:
```bash
STAKE='{"create_task":{"task":{"interval":"Immediate","boundary":{},"stop_on_fail":false,"actions":[{"msg":{"staking":{"delegate":{"validator":"juno14vhcdsyf83ngsrrqc92kmw8q9xakqjm0ff2dpn","amount":{"denom":"ujunox","amount":"10000"}}}},"gas_limit":150000}],"rules":null}}}'
junod tx wasm execute $CONTRACT "$STAKE" --amount 100000ujunox --from $USER $TXFLAG -y

STAKE3='{"create_task":{"task":{"interval":"Immediate","boundary":{},"stop_on_fail":false,"actions":[{"msg":{"staking":{"delegate":{"validator":"juno14vhcdsyf83ngsrrqc92kmw8q9xakqjm0ff2dpn","amount":{"denom":"ujunox","amount":"30000"}}}},"gas_limit":150000}],"rules":null}}}'
junod tx wasm execute $CONTRACT "$STAKE3" --amount 100000ujunox --from $USER $TXFLAG -y
```
`USER` can refill the third task:
```bash
REFILL_TASK_BALANCE='{"refill_task_balance":{"task_hash":"435c84ef3c6df933645a3f9c85e53dbd561ea0c9cf24838053514b8858fdb933"}}'
junod tx wasm execute $CONTRACT "$REFILL_TASK_BALANCE" --amount 200000ujunox --from $USER $TXFLAG -y
```
He also may remove the task:
```bash
REMOVE_TASK='{"remove_task":{"task_hash":"435c84ef3c6df933645a3f9c85e53dbd561ea0c9cf24838053514b8858fdb933"}}'
junod tx wasm execute $CONTRACT "$REMOVE_TASK" --from $USER $TXFLAG -y
```

## Agents
`AGENT` registers as agent. Since he is the first agent, he will be automatically put in the list of active agents.
```bash
REGISTER_AGENT='{"register_agent":{}}'
junod tx wasm execute $CONTRACT "$REGISTER_AGENT" --from $AGENT $TXFLAG -y
```
The agent can change the payment account:
```bash
UPDATE_AGENT='{"update_agent":{"payable_account_id":"'$(junod keys show $USER -a)'"}}'
junod tx wasm execute $CONTRACT "$UPDATE_AGENT" --from $AGENT $TXFLAG -y
```
`AGENT` should execute tasks by calling `proxy_call`
```bash
PROXY_CALL='{"proxy_call":{}}'
junod tx wasm execute $CONTRACT "$PROXY_CALL" --from $AGENT $TXFLAG -y
```
After executing tasks he can withdraw the tokens that he earned:
```bash
WITHDRAW_REWARD='{"withdraw_reward":{}}'
junod tx wasm execute $CONTRACT "$WITHDRAW_REWARD" --from $AGENT $TXFLAG -y
```
To withdraw the reward and unregister:
```bash
UNREGISTER_AGENT='{"unregister_agent":{}}'
junod tx wasm execute $CONTRACT "$UNREGISTER_AGENT" --from $AGENT $TXFLAG -y
```
When the second agent tries to register, he is put in pending list. 
If there are enough tasks, one can accept nomination and become an agent.
In our case the user needs to wait when one more task is added and accepts nomination.
```bash
REGISTER_AGENT='{"register_agent":{}}'
junod tx wasm execute $CONTRACT "$REGISTER_AGENT" --from $USER $TXFLAG -y

STAKE4='{"create_task":{"task":{"interval":"Immediate","boundary":{},"stop_on_fail":false,"actions":[{"msg":{"staking":{"delegate":{"validator":"juno14vhcdsyf83ngsrrqc92kmw8q9xakqjm0ff2dpn","amount":{"denom":"ujunox","amount":"400000"}}}},"gas_limit":150000}],"rules":null}}}'
junod tx wasm execute $CONTRACT "$STAKE4" --amount 100000ujunox --from $USER $TXFLAG -y

CHECK_IN_AGENT='{"check_in_agent":{}}'
junod tx wasm execute $CONTRACT "$CHECK_IN_AGENT" --from $USER $TXFLAG -y
```

## Owner
Only `OWNER` can update settings of the contract.
For example, here we pause it for creating and executing tasks, registering agents:
```bash
UPDATE_SETTINGS='{"update_settings":{"paused":true}}'
junod tx wasm execute $CONTRACT "$UPDATE_SETTINGS" --from $OWNER $TXFLAG -y
```
`OWNER`may move balances from contract to his address.
```bash
MOVE_BALANCES='{"move_balances":{"balances":[],"account_id":"'$(junod keys show $OWNER -a)'"}}'
junod tx wasm execute $CONTRACT "$MOVE_BALANCES" --from $OWNER $TXFLAG -y
```
## Query
To get the config:
```bash
GET_CONFIG='{"get_config":{}}'
junod query wasm contract-state smart $CONTRACT "$GET_CONFIG" $NODE
```
To get balances of the contract address:
```bash
GET_BALANCES='{"get_balances":{}}'
junod query wasm contract-state smart $CONTRACT "$GET_BALANCES" $NODE
```
To get agent details:
```bash
GET_AGENT='{"get_agent":{"account_id":"'$(junod keys show $USER -a)'"}}'
junod query wasm contract-state smart $CONTRACT "$GET_AGENT" $NODE
```
To get addresses of active and pending agents:
```bash
GET_AGENT_IDS='{"get_agent_ids":{}}'
junod query wasm contract-state smart $CONTRACT "$GET_AGENT_IDS" $NODE
```
To get info about tasks fot the agent:
```bash
GET_AGENT_TASKS='{"get_agent_tasks":{"account_id":"'$(junod keys show $USER -a)'"}}'
junod query wasm contract-state smart $CONTRACT "$GET_AGENT_TASKS" $NODE #doesn't work yet 
```
To get details of all tasks:
```bash
GET_TASKS='{"get_tasks":{}}'
junod query wasm contract-state smart $CONTRACT "$GET_TASKS" $NODE
```
To see tasks created by the specific user:
```bash
GET_TASKS_BY_OWNER='{"get_tasks_by_owner":{"owner_id":"'$(junod keys show $USER -a)'"}}'
junod query wasm contract-state smart $CONTRACT "$GET_TASKS_BY_OWNER" $NODE
```
To get a task by the hash:
```bash
GET_TASK='{"get_task":{"task_hash":"4905bb310073e83af6cd9c4c19f9f5782db79e7f8b08b4035b664d8f39d31dd7"}}'
junod query wasm contract-state smart $CONTRACT "$GET_TASK" $NODE
```
To get a hash of the task:
```bash
GET_TASK_HASH='{"get_task_hash":{"task":{"owner_id":"juno1qgdwpzngq8wtrd0xamfpr0fse7egrefye6ekuh","interval":"Immediate","boundary":{"start":null,"end":null},"stop_on_fail":false,"total_deposit":[{"denom":"ujunox","amount":"1"}],"actions":[{"msg":{"staking":{"delegate":{"validator":"juno14vhcdsyf83ngsrrqc92kmw8q9xakqjm0ff2dpn","amount":{"denom":"ujunox","amount":"400000"}}}},"gas_limit":150000}],"rules":null}}}'
junod query wasm contract-state smart $CONTRACT "$GET_TASK_HASH" $NODE
```
Check if the interval is valid:
```bash
VALIDATE_INTERVAL='{"validate_interval":{"interval":"Once"}}'
junod query wasm contract-state smart $CONTRACT "$VALIDATE_INTERVAL" $NODE
```
To get the next executable set of tasks hashes:

```bash
GET_SLOT_HASHES='{"get_slot_hashes":{}}'
junod query wasm contract-state smart $CONTRACT "$GET_SLOT_HASHES" $NODE

GET_SLOT_HASHES_SLOT='{"get_slot_hashes":{"slot":800000}}'
junod query wasm contract-state smart $CONTRACT "$GET_SLOT_HASHES_SLOT" $NODE
```
To gets list of active slot ids, for both time and block slots:
```bash
GET_SLOT_IDS='{"get_slot_ids":{}}'
junod query wasm contract-state smart $CONTRACT "$GET_SLOT_IDS" $NODE
```