#!/bin/bash
set -e
WASM_POSTFIX="-aarch64"
DIR=$(pwd)
JUNO_DIR="$HOME/juno"
DIR_NAME=$(basename "$PWD")
SCRIPT_PATH=$(dirname $(which $0))
NODE="--node https://rpc.uni.juno.deuslabs.fi:443"
TXFLAG="--node https://rpc.uni.juno.deuslabs.fi:443 --chain-id uni-5 --gas-prices 0.025ujunox --gas auto --gas-adjustment 1.3 --broadcast-mode block"

# Reset
NoColor='\033[0m' # Text Reset
# Regular Colors
Black='\033[0;30m'  # Black
Red='\033[0;31m'    # Red
Green='\033[0;32m'  # Green
Yellow='\033[0;33m' # {Yellow}
Blue='\033[0;34m'   # Blue
Purple='\033[0;35m' # Purple
Cyan='\033[0;36m'   # Cyan
White='\033[0;37m'  # White


cd ../../..

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  --platform linux/arm64 \
  cosmwasm/workspace-optimizer:0.12.8

if [ "$#" -eq 3 ]
then
    OWNER=$1
    AGENT=$2
    USER=$3
else 
    . ./contract/cw-croncat/scripts/init_addresses.sh
fi

# move binary to docker container
cd $DIR
docker cp "artifacts/$DIR_NAME_SNAKE$WASM_POSTFIX.wasm" "$IMAGE_NAME:/$DIR_NAME_SNAKE$WASM_POSTFIX.wasm"
docker cp "artifacts/cw_rules$WASM_POSTFIX.wasm" "$IMAGE_NAME:/cw_rules$WASM_POSTFIX.wasm"
docker cp "artifacts/cw20_base.wasm" "$IMAGE_NAME:/cw20_base.wasm"

echo "${Cyan}Wasm file: $WASM"
echo "${Cyan}Wasm file: cw_rules$WASM_POSTFIX.wasm"
echo "${Cyan}Wasm file: cw20_base.wasm"

cd $JUNO_DIR

# Instantiate
INIT='{"denom":"ujunox"}'
junod tx wasm instantiate $CODE_ID "$INIT" --from $OWNER --label "croncat" $TXFLAG -y --no-admin
CONTRACT=$(junod query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')

# Now we can register an agent, create tasks and execute a task
# Register an agent
REGISTER_AGENT='{"register_agent":{}}'
junod tx wasm execute $CONTRACT "$REGISTER_AGENT" --from $AGENT $TXFLAG -y

# Create a task
STAKE='{"create_task":{"task":{"interval":"Immediate","boundary":null,"cw20_coins":[],"stop_on_fail":false,"actions":[{"msg":{"staking":{"delegate":{"validator":"juno14vhcdsyf83ngsrrqc92kmw8q9xakqjm0ff2dpn","amount":{"denom":"ujunox","amount":"10000"}}}},"gas_limit":150000}],"rules":null}}}'
junod tx wasm execute $CONTRACT "$STAKE" --amount 10000ujunox --from $USER $TXFLAG -y

# proxy_call
sleep 10      # is needed to make sure this call in the next block 
PROXY_CALL='{"proxy_call":{}}'
junod tx wasm execute $CONTRACT "$PROXY_CALL" --from $AGENT $TXFLAG -y

echo "CONTRACT CODEID - $CODE_ID"
echo "CONTRACT $CONTRACT"
echo "OWNER $OWNER"
echo "AGENT $AGENT"
echo "USER $USER"
