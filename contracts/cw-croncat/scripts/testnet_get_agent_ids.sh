#!/bin/bash
. ./testnet_init_vars.sh

CONTRACT=juno12z4hh9r3j9aurjn6ppkgyjrkuu4ugrdectsh792w8feyj56dhlssvntdls
GET_AGENT_IDS='{"get_agent_ids":{}}'
junod query wasm contract-state smart $CONTRACT "$GET_AGENT_IDS" $NODE