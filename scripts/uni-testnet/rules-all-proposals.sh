#!/bin/bash
set -e

cd "$(dirname "$0")"
. ./base/init-vars.sh

if [ -z "$1" ]
then
    echo "Must provide rules contract address"
    exit 1
elif [ -z "$2" ]
then
    echo "Must provide dao address"
    exit 1
else
    RULES_CONTRACT="$1"
    DAO="$2"
fi

GET_PASSED_PROPOSALS='{
  "check_passed_proposals": {
    "dao_address": "'$DAO'"
  }
}'
junod query wasm contract-state smart $RULES_CONTRACT "$GET_PASSED_PROPOSALS" $NODE
