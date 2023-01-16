#!/bin/sh
set -ex
source ~/.profile
SH_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SH_DIR="$(cd -P "$(dirname "${SH_PATH}")";pwd)"
SC_PATH="$(cd -P "$(dirname "${SH_PATH}")/../..";pwd)"
SCRIPTS_PATH="$(cd -P "$(dirname "${SH_PATH}")/..";pwd)"

echo "CONTRACT-DIR: $SC_PATH"
echo "SCRIPT-DIR: $SH_DIR"
cd $SC_PATH

$SCRIPTS_PATH/build.sh
echo "Initializing vars"
. $SH_DIR/base/init-vars.sh

usage() {
  printf "Usage: $SH_DIR/simple-payroll.sh -w -c"
}
flags() {
  while test $# -gt 0; do
    case "$1" in
    -w | --recreate-artifacts)
      RECREATE_ARTIFACTS=1
      ;;
    -c | --recreate-containers)
      RECREATE_CONTAINERS=1
      ;;
    -a | --all)
      RECREATE_ARTIFACTS=1
      RECREATE_CONTAINERS=1
      ;;
    -\? | -h | --help)
      usage
      exit
      ;;
    --) # Stop option processing
      usage
      exit 1
      ;;
    -*)
      usage
      exit 1
      ;;
    *)
      usage
      exit 1
      ;;
    esac

    # and here we shift to the next argument
    shift
  done
}

if [[ -z "$@" ]]; then
  RECREATE_ARTIFACTS=0
  RECREATE_CONTAINERS=0
else
  flags "$@"
fi
if [[ -z "$RECREATE_ARTIFACTS" ]]; then
  RECREATE_ARTIFACTS=0
fi
if [[ -z "$RECREATE_CONTAINERS" ]]; then
  RECREATE_CONTAINERS=0
fi
echo "RECREATE_ARTIFACTS " $RECREATE_ARTIFACTS
echo "RECREATE_CONTAINERS " $RECREATE_CONTAINERS

#Recreate artifacts
if [ $RECREATE_ARTIFACTS == 1 ]; then
  #Remove local artifacts folder
  echo "deleting artifacts..."
  rm -rf "artifacts"
  # build optimized binary if it doesn't exist
  if [ ! -f "$WASM" ]; then
    echo "building optimized binary..."
    docker run --rm -v "$(pwd)":/code \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
      --platform linux/arm64 \
      cosmwasm/rust-optimizer$PLATFORM:0.12.8
  fi
  #Download basic implementation of a cw20
  curl -o artifacts/cw20_base.wasm -LO "https://github.com/CosmWasm/cw-plus/releases/download/v0.13.4/cw20_base.wasm"

fi
#Recreate containers
if [ $RECREATE_CONTAINERS == 1 ]; then
  . $SH_DIR/base/init-addresses.sh
fi


echo "${Cyan}Wasm file: $WASM"
echo "${Cyan}Wasm file: cw_rules$WASM_POSTFIX.wasm"
echo "${Cyan}Wasm file: cw20_base.wasm"

. $SH_DIR/base/balances.sh

#---------------------------------------------------------------------------

echo "${Yellow}Instantiating smart contracts...${NoColor}"
RES=$(junod tx wasm store artifacts/cw_croncat$WASM_POSTFIX.wasm --from owner $TXFLAG -y --output json -b block)
CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[1].value')
RULES_ID=$($BINARY tx wasm store artifacts/cw_rules$WASM_POSTFIX.wasm --from owner $TXFLAG --output json -y | jq -r '.logs[0].events[-1].attributes[1].value')
CW20_ID=$($BINARY tx wasm store artifacts/cw20_base.wasm --from owner $TXFLAG --output json -y | jq -r '.logs[0].events[-1].attributes[1].value')


echo "${Cyan}CODE_ID :" $CODE_ID "${NoColor}"
echo "${Cyan}RULES_ID :" $RULES_ID "${NoColor}"
echo "${Cyan}CW20_ID :" $CW20_ID "${NoColor}"

$BINARY tx wasm instantiate $RULES_ID '{}' --from owner --label "cw_rules" $TXFLAG -y --no-admin
RULES_CONTRACT_ADDR=$($BINARY q wasm list-contract-by-code $RULES_ID $NODE --output json | jq -r '.contracts[-1]')
echo "${Cyan}RULES_CONTRACT_ADDR :" $RULES_CONTRACT_ADDR "${NoColor}"

INIT_CW20='{"name": "memecoin", "symbol": "meme", "decimals": 4, "initial_balances": [{"address": "'$($BINARY keys show owner -a)'", "amount": "100000"}]}'
$BINARY tx wasm instantiate $CW20_ID "$INIT_CW20" --from owner --label "memecoin" $TXFLAG -y --no-admin
CW20_ADDR=$($BINARY q wasm list-contract-by-code $CW20_ID $NODE --output json | jq -r '.contracts[-1]')
echo "${Cyan}CW20_ADDR :" $CW20_ADDR "${NoColor}"

#Croncat
INIT='{"denom":"'$STAKE'","cw_rules_addr":"'$RULES_CONTRACT_ADDR'"}'
$BINARY tx wasm instantiate $CODE_ID "$INIT" --from owner --label "croncat" $TXFLAG -y --no-admin

# get smart contract address
CONTRACT_ADDRESS=$($BINARY query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')
echo "${Cyan}CONTRACT_ADDRESS :" $CONTRACT_ADDRESS "${NoColor}"
echo "${Cyan}Instantiating smart contracts done!${NoColor}"

#Display all data
echo "${Cyan}"
echo ALICE_ADDR=$ALICE_ADDR
echo BOB_ADDR=$BOB_ADDR
echo OWNER_ADDR=$OWNER_ADDR
echo USER_ADDR=$USER_ADDR
echo AGENT_ADDR=$AGENT_ADDR
echo RULES_CONTRACT_ADDR=$RULES_CONTRACT_ADDR
echo CW20_ADDR=$CW20_ADDR
echo CONTRACT_ADDRESS=$CONTRACT_ADDRESS"${NoColor}"
