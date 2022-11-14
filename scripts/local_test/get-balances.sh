#!/bin/sh
set -e

cd "$(dirname "$0")"
. ./local_init_vars.sh

ALICE_BALANCE=$($BINARY q bank balances $($BINARY keys show alice --address) --denom ujunox)
echo "${Green}Alice Balance:" $ALICE_BALANCE "${NoColor}"

BOB_BALANCE=$($BINARY q bank balances $($BINARY keys show bob --address) --denom ujunox)
echo "${Green}Bob Balance:" $BOB_BALANCE "${NoColor}"

USER_BALANCE=$($BINARY q bank balances $($BINARY keys show user --address) --denom ujunox)
echo "${Green}User Balance:" $USER_BALANCE "${NoColor}"

AGENT_BALANCE=$($BINARY q bank balances $($BINARY keys show agent --address) --denom ujunox)
echo "${Green}Agent Balance:" $AGENT_BALANCE "${NoColor}"

VALIDATOR=$($BINARY q bank balances $($BINARY keys show validator --address) --denom ujunox)
echo "${Green}Validator Balance:" $VALIDATOR "${NoColor}"

CONTRACT_BALANCE=$($BINARY q bank balances $CONTRACT_ADDRESS --denom ujunox)
echo "${Green}Contract Balance:" $CONTRACT_BALANCE "${NoColor}"

# $BINARY tx bank send validator $CONTRACT_ADDRESS "1000ujunox" --from validator --yes --broadcast-mode block --sign-mode direct --chain-id $CHAIN_ID

CONTRACT_BALANCE=$($BINARY q bank balances $CONTRACT_ADDRESS --denom ujunox)
echo "${Green}Contract Balance:" $CONTRACT_BALANCE "${NoColor}"
