#!/bin/bash
set -e
# Destroy local juno
rm -rf ~/.juno
# Destroy local Croncat agent settings, including agent(s) keypairs
rm -rf ~/.croncatd

cd "$(dirname "$0")"
if [ -z "$1" ]
then
    echo "Must provide cw-rules contract address"
    exit 1
else 
    cw_rules_addr=$1
fi
. ./init-vars.sh

junod init croncat --chain-id croncat-0.0.1 --overwrite
sleep 2

# Thanks to
#   Ethan Frey https://github.com/CosmWasm/wasmd/commit/810c05bbcadf903b687f1365f3927cb65511dc1f#diff-d37e3d2a27ee91db29137c81077c90b138427843e8f21e78f0c6e7645803ad1cR14
#   and Jorge Hernandez's message here: https://discord.com/channels/737637324434833438/737640672680607764/1019038743610523730
sed -i '' -e 's/"time_iota_ms": "1000"/"time_iota_ms": "10"/' "$HOME"/.juno/config/genesis.json
sed -i '' -e 's/timeout_commit = "5s"/timeout_commit = "1s"/' "$HOME"/.juno/config/config.toml
sed -i '' -e 's/timeout_propose = "5s"/timeout_propose = "1s"/' "$HOME"/.juno/config/config.toml

VALIDATOR_SEED_PHRASE="before ice gravity winner mystery noble rug science barely patrol snake foot jelly buddy olympic remove addict health whale better purse pen vacant attract"
junod keys show validator 2> /dev/null || echo $VALIDATOR_SEED_PHRASE | junod keys add validator --recover
OWNER_SEED_PHRASE="bind desert siege network dog any fix carbon evidence install any eternal front hidden you report still basic nothing market mask youth early cigar"
junod keys show owner 2> /dev/null || echo $OWNER_SEED_PHRASE | junod keys add owner --recover
AGENT_SEED_PHRASE="shove click bless section used eye able chaos welcome peasant base apart issue reduce sphere oven salmon glow distance strategy tortoise spot grunt area"
junod keys show agent 2> /dev/null || echo $AGENT_SEED_PHRASE | junod keys add agent --recover # && croncatd generate-mnemonic --mnemonic $AGENT_SEED_PHRASE
USER_SEED_PHRASE="gas silly unlock shy face bless pave fancy hamster snap coast scare kingdom reopen deny make pride sea shine night curve source cram bunker"
junod keys show user 2> /dev/null || echo $USER_SEED_PHRASE | junod keys add user --recover
ALICE_SEED_PHRASE="fix salute raise you copper outer illness mosquito version cave broccoli stick limit glad typical harsh retreat rebuild unhappy settle guilt churn slam chalk"
junod keys show alice 2> /dev/null || echo $ALICE_SEED_PHRASE | junod keys add alice --recover
BOB_SEED_PHRASE="book obey ensure swarm ill drink blind process trend certain kind enhance motion world flame portion select crater fruit tuition brick earth fee weird"
junod keys show bob 2> /dev/null || echo $BOB_SEED_PHRASE | junod keys add bob --recover

junod add-genesis-account $(junod keys show validator -a) 10000000000000000000000000stake
junod gentx validator 1000000000000000stake --chain-id croncat-0.0.1 --chain-id croncat-0.0.1
junod collect-gentxs
sleep 1

# Start the Juno chain, making sure we have gRPC
junod start --grpc.address "127.0.0.1:9090" >/dev/null 2>&1 < /dev/null &
sleep 3

# Send funds from the validator to necessary parties
FUND_RES=$(junod tx bank send $(junod keys show validator -a) $(junod keys show owner -a) 600000000000stake --chain-id croncat-0.0.1 --sequence 1 -y)
junod tx bank send $(junod keys show validator -a) $(junod keys show agent -a) 600000000000stake --chain-id croncat-0.0.1 --sequence 2 -y
junod tx bank send $(junod keys show validator -a) $(junod keys show user -a) 600000000000stake --chain-id croncat-0.0.1  --sequence 3 -y
junod tx bank send $(junod keys show validator -a) $(junod keys show agent -a) 600000000000stake --chain-id croncat-0.0.1  --sequence 4 -y
junod tx bank send $(junod keys show validator -a) $(junod keys show alice -a) 1stake --chain-id croncat-0.0.1  --sequence 5 -y
junod tx bank send $(junod keys show validator -a) $(junod keys show bob -a) 1stake --chain-id croncat-0.0.1  --sequence 6 -y
echo "Fund owner result: $FUND_RES"
sleep 1
# Upload the Croncat Manager contract
echo "wasm store cw_croncat.wasm..."
RES=$(junod tx wasm store ../../artifacts/cw_croncat.wasm --from owner --node http://localhost:26657 --chain-id croncat-0.0.1 --gas-prices 0.025stake --gas auto --gas-adjustment 1.3 --broadcast-mode block -y --output json -b block)
CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value')
echo "Code ID: $CODE_ID"

# Instantiate
echo "Instantiating cw_croncat contract..."

INIT='{"denom":"stake","cw_rules_addr":"$cw_rules_addr"}'
junod tx wasm instantiate $CODE_ID "$INIT" --from owner --label "croncat" $TXFLAG --no-admin -y
CONTRACT=$(junod query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')
echo "Croncat Manager contract address: $CONTRACT"

echo "Creating simple payroll"
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
                    "denom": "stake"
                  }
                ],
                "to_address": "'$(junod keys show alice -a)'"
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
                    "denom": "stake"
                  }
                ],
                "to_address": "'$(junod keys show bob -a)'"
              }
            }
          }
        }
      ],
      "rules": []
    }
  }
}'
junod tx wasm execute $CONTRACT "$SIMPLE_PAYROLL" --amount 1000000000stake --from user $TXFLAG -y
echo "Done creating simple payroll"
