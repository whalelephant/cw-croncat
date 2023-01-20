#!/bin/sh
set -e
source ~/.profile
SH_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SH_DIR="$(cd -P "$(dirname "${SH_PATH}")";pwd)"
SC_PATH="$(cd -P "$(dirname "${SH_PATH}")/../..";pwd)"
SCRIPTS_PATH="$(cd -P "$(dirname "${SH_PATH}")/..";pwd)"
WASM_AGENTS="artifacts/croncat_agents.wasm"
echo "CONTRACT-DIR: $SC_PATH"
echo "SCRIPT-DIR: $SH_DIR"
cd $SC_PATH

echo "Initializing vars"
. $SH_DIR/common/dec.sh

usage() {
  printf "Usage: $SH_DIR/start.sh -w -c"
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
  if [ ! -f "$WASM_AGENTS" ]; then
    echo "building optimized binary..."
    docker run --rm -v "$(pwd)":/code \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
      cosmwasm/rust-optimizer$PLATFORM:0.12.11
  fi
  #Download basic implementation of a cw20
  curl -o artifacts/cw20_base.wasm -LO "https://github.com/CosmWasm/cw-plus/releases/download/v0.13.4/cw20_base.wasm"

fi
#Recreate containers
if [ $RECREATE_CONTAINERS == 1 ]; then
  . $SH_DIR/common/decad.sh
fi


echo "${Cyan}Wasm file: $WASM_AGENTS"
echo "${Cyan}Wasm file: cw20_base.wasm"

. $SH_DIR/common/infobal.sh

#---------------------------------------------------------------------------

echo "${Yellow}Instantiating smart contracts...${NoColor}"
AGENTS_RES=$(junod tx wasm store $WASM_AGENTS --from owner $TXFLAG -y --output json -b block)
AGENTS_CODE_ID=$(echo $AGENTS_RES | jq -r '.logs[0].events[-1].attributes[1].value')

echo "${Cyan}CODE_ID :" $AGENTS_CODE_ID "${NoColor}"

#AGENTS
AGENTS_INIT='{}'
$BINARY tx wasm instantiate $CODE_ID "$AGENTS_INIT" --from owner --label "croncat-agents" $TXFLAG -y --no-admin

# get smart contract address
AGENTS_CONTRACT_ADDRESS=$($BINARY query wasm list-contract-by-code $AGENTS_CODE_ID $NODE --output json | jq -r '.contracts[-1]')
echo "${Cyan}AGENTS_CONTRACT_ADDRESS :" $AGENTS_CONTRACT_ADDRESS "${NoColor}"
echo "${Cyan}Instantiating smart contracts done!${NoColor}"

#Display all data
echo "${Cyan}"
echo ALICE_ADDR=$ALICE_ADDR
echo BOB_ADDR=$BOB_ADDR
echo OWNER_ADDR=$OWNER_ADDR
echo USER_ADDR=$USER_ADDR
echo AGENT_ADDR=$AGENT_ADDR
echo AGENTS_CONTRACT_ADDRESS=$AGENTS_CONTRACT_ADDRESS"${NoColor}"