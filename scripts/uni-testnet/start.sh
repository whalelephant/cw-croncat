#!/bin/sh
set -e

./scripts/build.sh

./scripts/uni-testnet/base/init_vars.sh


usage() {
  printf "Usage: ./scripts/uni-testnet/simple-payroll.sh -w -c"
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
  usage
  exit
else
  flags "$@"
fi
echo "RECREATE_ARTIFACTS " $RECREATE_ARTIFACTS
echo "RECREATE_CONTAINERS " $RECREATE_CONTAINERS
echo $RECREATE_CONTAINERS
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
  . ./init_addresses.sh
fi

# move binary to docker container
cd $DIR
docker cp "artifacts/$DIR_NAME_SNAKE$WASM_POSTFIX.wasm" "$IMAGE_NAME:/$DIR_NAME_SNAKE$WASM_POSTFIX.wasm"
docker cp "artifacts/cw_rules$WASM_POSTFIX.wasm" "$IMAGE_NAME:/cw_rules$WASM_POSTFIX.wasm"
docker cp "artifacts/cw20_base.wasm" "$IMAGE_NAME:/cw20_base.wasm"

echo "${Cyan}Wasm file: $WASM"
echo "${Cyan}Wasm file: cw_rules$WASM_POSTFIX.wasm"
echo "${Cyan}Wasm file: cw20_base.wasm"

. ./base/balances.sh

#---------------------------------------------------------------------------
echo "${Yellow}Instantiating smart contracts...${NoColor}"
CODE_ID=$($BINARY tx wasm store /$DIR_NAME_SNAKE$WASM_POSTFIX.wasm --from $OWNER $TXFLAG --output json | jq -r '.logs[0].events[-1].attributes[0].value')
RULES_ID=$($BINARY tx wasm store "/cw_rules$WASM_POSTFIX.wasm" --from $OWNER $TXFLAG --output json | jq -r '.logs[0].events[-1].attributes[0].value')
CW20_ID=$($BINARY tx wasm store "/cw20_base.wasm" --from $OWNER $TXFLAG --output json | jq -r '.logs[0].events[-1].attributes[0].value')

echo "${Cyan}CODE_ID :" $CODE_ID "${NoColor}"
echo "${Cyan}RULES_ID :" $RULES_ID "${NoColor}"
echo "${Cyan}CW20_ID :" $CW20_ID "${NoColor}"

$BINARY tx wasm instantiate $RULES_ID '{}' --from $OWNER --label "cw_rules" $TXFLAG -y --no-admin
RULES_CONTRACT_ADDR=$($BINARY q wasm list-contract-by-code $RULES_ID --output json | jq -r '.contracts[-1]')
echo "${Cyan}RULES_CONTRACT_ADDR :" $RULES_CONTRACT_ADDR "${NoColor}"

INIT_CW20='{"name": "memecoin", "symbol": "meme", "decimals": 4, "initial_balances": [{"address": "'$($BINARY keys show $OWNER -a)'", "amount": "100000"}]}'
$BINARY tx wasm instantiate $CW20_ID "$INIT_CW20" --from $OWNER --label "memecoin" $TXFLAG -y --no-admin
CW20_ADDR=$($BINARY q wasm list-contract-by-code $CW20_ID --output json | jq -r '.contracts[-1]')
echo "${Cyan}CW20_ADDR :" $CW20_ADDR "${NoColor}"

#Croncat
INIT='{"denom":"'$STAKE'","cw_rules_addr":"'$RULES_CONTRACT_ADDR'"}'
$BINARY tx wasm instantiate $CODE_ID "$INIT" --from $OWNER --label "croncat" $TXFLAG -y --no-admin

# get smart contract address
CONTRACT_ADDRESS=$($BINARY query wasm list-contract-by-code $CODE_ID --output json | jq -r '.contracts[-1]')
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
echo CONTRACT_ADDRESS=$CONTRACT_ADDRESS$ "{NoColor}"
