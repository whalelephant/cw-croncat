#!/bin/bash
set -e
. ./init_vars.sh

OWNER_BALANCE=12000000
AGENT_BALANCE=10000
USER_BALANCE=15000

OWNER=cw-croncat-test-owner
AGENT=cw-croncat-test-agent
USER=cw-croncat-test-user

$BINARY keys show $OWNER 2> /dev/null || $BINARY keys add $OWNER
$BINARY keys show $AGENT 2> /dev/null || $BINARY keys add $AGENT
$BINARY keys show $USER 2> /dev/null || $BINARY keys add $USER

FAUCET_SEED_PHRASE="very priority voice drink cloud advance wait pave dose useful erode proud just absorb east eyebrow unaware prize old brand above arrow east aim"
$BINARY keys show cw-croncat-faucet 2> /dev/null || echo $FAUCET_SEED_PHRASE | $BINARY keys add cw-croncat-faucet --recover

echo "${Yellow}Sending funds to users...${NoColor}"
$BINARY tx bank send cw-croncat-faucet $($BINARY keys show "$OWNER" -a) "$OWNER_BALANCE"ujunox $NODE --chain-id $CHAIN_ID
$BINARY tx bank send cw-croncat-faucet $($BINARY keys show "$AGENT" -a) "$AGENT_BALANCE"ujunox $NODE --chain-id $CHAIN_ID
$BINARY tx bank send cw-croncat-faucet $($BINARY keys show "$USER" -a) "$USER_BALANCE"ujunox $NODE --chain-id $CHAIN_ID
echo "${Cyan}Funds sent...${NoColor}"

$BINARY query bank balances $($BINARY keys show cw-croncat-faucet -a) $NODE
$BINARY query bank balances $($BINARY keys show "$OWNER" -a) $NODE
$BINARY query bank balances $($BINARY keys show "$AGENT" -a) $NODE
$BINARY query bank balances $($BINARY keys show "$USER" -a) $NODE


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