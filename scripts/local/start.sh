#!/bin/sh
#Usage exmaple
#Parameters:
# -w --reset_wasm
# -c --reset_containers
#sudo ./scripts/local/start-in-docker.sh   -w -c
set -e
just build
CHAIN_ID="testing"
RPC="http://localhost:26657/"
BINARY="docker exec -i juno-node-1 junod"
PLATFORM="-arm64"
WASM_POSTFIX="-aarch64"
SH_DIR="$(
    cd -P "$(dirname "${SH_PATH}")"
    pwd
)"
JUNO_DIR="$HOME/juno"
DIR_NAME=$(basename "$PWD")
SH_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SC_PATH="$(
    cd -P "$(dirname "${SH_PATH}")/../.."
    pwd
)"

SH_DIR="$(
    cd -P "$(dirname "${SH_PATH}")"
    pwd
)"
IMAGE_NAME="juno-node-1"
DIR_NAME_SNAKE=$(echo $DIR_NAME | tr '-' '_')
STAKE_TOKEN=ujunox
STAKE=${STAKE_TOKEN:-ustake}
TXFLAG="--gas-prices 0.025ujunox --gas auto --gas-adjustment 1.3 -y -b block --chain-id $CHAIN_ID --node $RPC"
RECREATE_ARTIFACTS=0
RECREATE_CONTAINERS=0

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

cd $SC_PATH

usage() {
    printf '%s\n' "Usage: ./scripts/local/start.sh -w -c"
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

flags "$@"

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
            --platform linux/amd64 \
            cosmwasm/workspace-optimizer:0.12.10
    fi
    # #Download basic implementation of a cw20
    # curl -o artifacts/cw20_base.wasm -LO "https://github.com/CosmWasm/cw-plus/releases/download/v0.13.4/cw20_base.wasm"

fi
#Recreate containers
if [ $RECREATE_CONTAINERS == 1 ]; then
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

    sleep 10
fi

# echo $DIR
# # move binary to docker container
docker cp $SC_PATH"/artifacts/croncat_agents$WASM_POSTFIX.wasm" "$IMAGE_NAME:/croncat_agents$WASM_POSTFIX.wasm"

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

# errors from this point

#$BINARY tx staking create-validator --chain-id $CHAIN_ID --from=validator --fees=0.025juno --pubkey=$VALIDATOR_ADDR --commission-rate=0.1 --amount=100000000ujuno --moniker=vname
#$BINARY gentx validator 15000000STAKE --chain-id $CHAIN_ID
#$BINARY collect-gentxs

# echo "${Yellow}Sending funds to users...${NoColor}"

# $BINARY tx bank send $VALIDATOR_ADDR $ALICE_ADDR "1$STAKE" --from validator --yes --broadcast-mode block --sign-mode direct --chain-id $CHAIN_ID
# $BINARY tx bank send $VALIDATOR_ADDR $BOB_ADDR "1$STAKE" --from validator --yes --broadcast-mode block --sign-mode direct --chain-id $CHAIN_ID
# $BINARY tx bank send $VALIDATOR_ADDR $OWNER_ADDR "60000000$STAKE" --from validator --yes --broadcast-mode block --sign-mode direct --chain-id $CHAIN_ID
# $BINARY tx bank send $VALIDATOR_ADDR $AGENT_ADDR "2000000$STAKE" --from validator --yes --broadcast-mode block --sign-mode direct --chain-id $CHAIN_ID
# $BINARY tx bank send $VALIDATOR_ADDR $USER_ADDR "40000000$STAKE" --from validator --yes --broadcast-mode block --sign-mode direct --chain-id $CHAIN_ID

# sleep 2
# echo "${Cyan}Funds sent...${NoColor}"

# ALICE_BALANCE=$($BINARY q bank balances $($BINARY keys show alice --address))
# echo "${Green}Alice Balance :" $ALICE_BALANCE "${NoColor}"
# BOB_BALANCE=$($BINARY q bank balances $($BINARY keys show bob --address))
# echo "${Green}Bob Balance :" $BOB_BALANCE "${NoColor}"
# OWNER_BALANCE=$($BINARY q bank balances $($BINARY keys show owner --address))
# echo "${Green}Owner Balance :" $OWNER_BALANCE "${NoColor}"
# AGENT_BALANCE=$($BINARY q bank balances $($BINARY keys show agent --address))
# echo "${Green}Agent Balance :" $AGENT_BALANCE "${NoColor}"
# USER_BALANCE=$($BINARY q bank balances $($BINARY keys show user --address))
# echo "${Green}User Balance :" $USER_BALANCE "${NoColor}"

#---------------------------------------------------------------------------

# cd $SC_PATH

# echo $AGENTS_WASM

# echo "${Yellow}Instantiating smart contracts...${NoColor}"
# AGENTS_CODE_ID=$($BINARY tx wasm store /croncat_agents$WASM_POSTFIX.wasm --from validator $TXFLAG --output json | jq -r '.logs[0].events[-1].attributes[0].value')
# echo "${Cyan}AGENTS_CODE_ID :" $AGENTS_CODE_ID "${NoColor}"

# #Croncat
# echo $OWNER_ADDR
# INIT='{"owner_addr":"'$OWNER_ADDR'","native_denom":"'$STAKE'"}'
# $BINARY tx wasm instantiate $AGENTS_CODE_ID "$INIT" --from owner --label "croncat" $TXFLAG -y --no-admin

# # get smart contract address
# AGENTS_CONTRACT_ADDRESS=$($BINARY query wasm list-contract-by-code $AGENTS_CODE_ID --output json | jq -r '.contracts[-1]')
# echo "${Cyan}AGENTS_CONTRACT_ADDRESS :" $AGENTS_CONTRACT_ADDRESS "${NoColor}"
# echo "${Cyan}Instantiating smart contracts done!${NoColor}"

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
