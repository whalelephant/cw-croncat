#!/bin/bash
. ./testnet_init_vars.sh
CONTRACT=juno12z4hh9r3j9aurjn6ppkgyjrkuu4ugrdectsh792w8feyj56dhlssvntdls
AGENT=juno1pd43m659naajmn2chkt6tna0uud2ywyp5dm4h3

GET_AGENT_TASKS='{"get_agent_tasks":{"account_id":"'$(junod keys show $AGENT -a)'"}}'
junod query wasm contract-state smart $CONTRACT "$GET_AGENT_TASKS" $NODE