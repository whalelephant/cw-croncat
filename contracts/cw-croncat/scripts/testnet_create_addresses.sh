#!/bin/bash
set -ex

cd "$(dirname "$0")"
. ./testnet_init_vars.sh

# Check if the balances were provided
if [ "$#" -eq 3 ]
then
    OWNER_BALANCE=$1
    AGENT_BALANCE=$2
    USER_BALANCE=$3
else 
    OWNER_BALANCE=30000
    AGENT_BALANCE=20000
    USER_BALANCE=10000
fi

OWNER=cw-croncat-test-owner
AGENT=cw-croncat-test-agent
USER=cw-croncat-test-user

junod keys show $OWNER 2> /dev/null || junod keys add $OWNER
junod keys show $AGENT 2> /dev/null || junod keys add $AGENT
junod keys show $USER 2> /dev/null || junod keys add $USER

FAUCET_SEED_PHRASE="very priority voice drink cloud advance wait pave dose useful erode proud just absorb east eyebrow unaware prize old brand above arrow east aim"
junod keys show cw-croncat-faucet 2> /dev/null || echo $FAUCET_SEED_PHRASE | junod keys add cw-croncat-faucet --recover

junod tx bank send cw-croncat-faucet $(junod keys show "$OWNER" -a) "$OWNER_BALANCE"ujunox $NODE --chain-id uni-3
junod tx bank send cw-croncat-faucet $(junod keys show "$AGENT" -a) "$AGENT_BALANCE"ujunox $NODE --chain-id uni-3
junod tx bank send cw-croncat-faucet $(junod keys show "$USER" -a) "$USER_BALANCE"ujunox $NODE --chain-id uni-3

junod query bank balances $(junod keys show cw-croncat-faucet -a) $NODE
junod query bank balances $(junod keys show "$OWNER" -a) $NODE
junod query bank balances $(junod keys show "$AGENT" -a) $NODE
junod query bank balances $(junod keys show "$USER" -a) $NODE
