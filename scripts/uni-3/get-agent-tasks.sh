#!/bin/bash

cd "$(dirname "$0")"
. ./init-vars.sh

if [ -z "$1" ]
then
    echo "Must provide contract address"
    exit 1
elif [ -z "$2" ]
then
    echo "Must provide agent address"
    exit 1
else
    CONTRACT="$1"
    AGENT="$2"
fi

GET_AGENT_TASKS='{"get_agent_tasks":{"account_id":"'$(junod keys show $AGENT -a)'"}}'
junod query wasm contract-state smart $CONTRACT "$GET_AGENT_TASKS" $NODE
