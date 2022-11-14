BINARY="docker exec -i juno_node_1 junod"

CHAIN_ID="testing"
RPC="http://localhost:26657/"
NODE="--node http://localhost:26657/"
STAKE_TOKEN=ujunox
STAKE=${STAKE_TOKEN:-ustake}
TXFLAG="--gas-prices 0.1$STAKE --gas auto --gas-adjustment 1.3 -y -b block --chain-id $CHAIN_ID --node $RPC"

Green='\033[0;32m'
White='\033[0;37m'
NoColor='\033[0m'

CONTRACT_ADDRESS=juno14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9skjuwg8