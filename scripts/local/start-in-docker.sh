#!/bin/sh
#Usage exmaple
#Parameters:
# --reset_artifacts --reset_container
#sudo ./scripts/local/start-in-docker.sh juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y  -no -yes
set -e

# # script for lauching local juno network
# if [ "$1" = "" ]; then
#   echo "Usage: $0 1 arg required - rules address"
#   exit
#   else
#     RULES_CONTRACT_ADDR=$1
# fi

CHAIN_ID="testing"
RPC="http://localhost:26657/"
BINARY="docker exec -i juno-node-1 junod"
DIR=$(pwd)
JUNO_DIR="$HOME/juno"
DIR_NAME=$(basename "$PWD")
SCRIPT_PATH=$(dirname `which $0`)
IMAGE_NAME="juno-node-1"
DIR_NAME_SNAKE=$(echo $DIR_NAME | tr '-' '_')
WASM="artifacts/$DIR_NAME_SNAKE.wasm"
STAKE_TOKEN=ujunox
STAKE=${STAKE_TOKEN:-ustake}
TXFLAG="--gas-prices 0.075$STAKE --gas auto --gas-adjustment 1.3 -y -b block --chain-id $CHAIN_ID --node $RPC"

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

#Recreate artifacts
if [ "$2" = "-yes" ]; then
  #Remove local artifacts folder
  echo "deleting artifacts..."
  rm -rf "artifacts"
  # build optimized binary if it doesn't exist
  if [ ! -f "$WASM" ]; then
    echo "building optimized binary..."
    docker run --rm -v "$(pwd)":/code \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
      cosmwasm/rust-optimizer:0.12.8
  fi
fi
#Recreate containers
if [ "$3" = "-yes" ]; then
  # stop docker container
  cd $JUNO_DIR
  echo "stopping container..."
  docker compose down
  # delete docker container
  echo "deleting container"
  docker rm -f $IMAGE_NAME 2>/dev/null

  # build new docker container
  echo "${Yellow}Starting local network${NoColor}"
  STAKE_TOKEN=ujunox UNSAFE_CORS=true docker-compose up -d
  echo "Adding new users..."

  # add new users
  ALICE_SEED="legend thunder embrace elegant tonight kid misery tragic merry design produce distance island city cancel shrimp dry eager shop scrub wait cigar tenant carry"
  echo $ALICE_SEED | $BINARY keys add alice --recover

  BOB_SEED="market rent damage chief intact require company female van scout accident amazing thought patch hammer any arch stereo aerobic plastic ranch fluid maple place"
  echo $BOB_SEED | $BINARY keys add bob --recover

  OWNER_SEED="scan quarter purchase hub enlist decade pumpkin young wisdom maple comic tooth surprise caution toe music universe skirt lady income decline sun steel pyramid"
  echo $OWNER_SEED | $BINARY keys add owner --recover

  AGENT_SEED="olive soup parade family educate congress hurt dwarf mom this position hungry unaware aunt swamp sunny analyst wrestle fashion main knife start coffee air"
  echo $AGENT_SEED | $BINARY keys add agent --recover

  USER_SEED="fatigue runway knock radio sauce express poem novel will ski various merge dolphin actor immune sea muffin decade pass exclude staff require hazard toe"
  echo $USER_SEED | $BINARY keys add user --recover

  sleep 5
fi

# move binary to docker container
cd $DIR
docker cp "artifacts/$DIR_NAME_SNAKE.wasm" "$IMAGE_NAME:/$DIR_NAME_SNAKE.wasm"
echo "${Cyan}Wasm file: $DIR_NAME_SNAKE"
cd $JUNO_DIR

# wait for chain starting before contract storing

# send them some coins
VALIDATOR_ADDR=$($BINARY keys show validator --address)

VALIDATOR_BALANCE=$($BINARY q bank balances $($BINARY keys show validator --address))
echo "${Cyan}Validator :" $VALIDATOR_ADDR "${NoColor}"
echo "${Green}Validator Balance :" $VALIDATOR_BALANCE "${NoColor}"

ALICE_ADDR=$($BINARY keys show alice --address)
BOB_ADDR=$($BINARY keys show bob --address)
OWNER_ADDR=$($BINARY keys show owner --address)
AGENT_ADDR=$($BINARY keys show agent --address)
USER_ADDR=$($BINARY keys show user --address)
echo "${Cyan}Alice :" $ALICE_ADDR "${NoColor}"
echo "${Cyan}Bob :" $BOB_ADDR "${NoColor}"
echo "${Cyan}Owner :" $OWNER_ADDR "${NoColor}"
echo "${Cyan}User :" $USER_ADDR "${NoColor}"
echo "${Cyan}Agent :" $AGENT_ADDR "${NoColor}"

if [ "$RULES_CONTRACT_ADDR" = "" ]; then
  RULES_CONTRACT_ADDR=$VALIDATOR_ADDR
fi

# errors from this point

#$BINARY tx staking create-validator --chain-id $CHAIN_ID --from=validator --fees=0.025juno --pubkey=$VALIDATOR_ADDR --commission-rate=0.1 --amount=100000000ujuno --moniker=vname
#$BINARY gentx validator 15000000STAKE --chain-id $CHAIN_ID
#$BINARY collect-gentxs

echo "${Yellow}Sending funds to users...${NoColor}"

$BINARY tx bank send $VALIDATOR_ADDR $ALICE_ADDR "1$STAKE" --from validator --yes --broadcast-mode block --sign-mode direct --chain-id $CHAIN_ID
$BINARY tx bank send $VALIDATOR_ADDR $BOB_ADDR "1$STAKE" --from validator --yes --broadcast-mode block --sign-mode direct --chain-id $CHAIN_ID
$BINARY tx bank send $VALIDATOR_ADDR $OWNER_ADDR "60000000$STAKE" --from validator --yes --broadcast-mode block --sign-mode direct --chain-id $CHAIN_ID
$BINARY tx bank send $VALIDATOR_ADDR $AGENT_ADDR "2000000$STAKE" --from validator --yes --broadcast-mode block --sign-mode direct --chain-id $CHAIN_ID
$BINARY tx bank send $VALIDATOR_ADDR $USER_ADDR "40000000$STAKE" --from validator --yes --broadcast-mode block --sign-mode direct --chain-id $CHAIN_ID

sleep 2
echo "${Cyan}Funds sent...${NoColor}"

ALICE_BALANCE=$($BINARY q bank balances $($BINARY keys show alice --address))
echo "${Green}Alice Balance :" $ALICE_BALANCE "${NoColor}"
BOB_BALANCE=$($BINARY q bank balances $($BINARY keys show bob --address))
echo "${Green}Bob Balance :" $BOB_BALANCE "${NoColor}"
OWNER_BALANCE=$($BINARY q bank balances $($BINARY keys show owner --address))
echo "${Green}Owner Balance :" $OWNER_BALANCE "${NoColor}"
AGENT_BALANCE=$($BINARY q bank balances $($BINARY keys show agent --address))
echo "${Green}Agent Balance :" $AGENT_BALANCE "${NoColor}"
USER_BALANCE=$($BINARY q bank balances $($BINARY keys show user --address))
echo "${Green}User Balance :" $USER_BALANCE "${NoColor}"

#---------------------------------------------------------------------------
echo "${Yellow}Instantiating smart contract...${NoColor}"
IRES=$($BINARY tx wasm store /$DIR_NAME_SNAKE.wasm --from validator $TXFLAG --output json)
CODE_ID=$(echo $IRES | jq -r '.logs[0].events[-1].attributes[0].value')
echo "${Cyan}CODE_ID :" $CODE_ID "${NoColor}"

INIT='{"denom":"'$STAKE'","cw_rules_addr":"'$RULES_CONTRACT_ADDR'"}'
echo "${Cyan} Rules Contract Addr:" $RULES_CONTRACT_ADDR "${NoColor}"

$BINARY tx wasm instantiate $CODE_ID "$INIT" --from owner --label "croncat" $TXFLAG -y --no-admin

# get smart contract address
CONTRACT_ADDRESS=$($BINARY query wasm list-contract-by-code $CODE_ID --output json | jq -r '.contracts[-1]')
echo "${Cyan}CONTRACT_ADDRESS :" $CONTRACT_ADDRESS "${NoColor}"
echo "${Cyan}Instantiating smart contract done!${NoColor}"
