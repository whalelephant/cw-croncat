#!/bin/bash
. ./testnet_init_vars.sh
USER=juno1pd43m659naajmn2chkt6tna0uud2ywyp5dm4h3
CONTRACT=juno12z4hh9r3j9aurjn6ppkgyjrkuu4ugrdectsh792w8feyj56dhlssvntdls

STAKE='{"create_task":{"task":{"interval":"Immediate","boundary":null,"cw20_coins":[],"stop_on_fail":false,"actions":[{"msg":{"staking":{"delegate":{"validator":"juno14vhcdsyf83ngsrrqc92kmw8q9xakqjm0ff2dpn","amount":{"denom":"ujunox","amount":"1000000"}}}},"gas_limit":150000}],"rules":null}}}'
junod tx wasm execute $CONTRACT "$STAKE" --amount 500000ujunox --from $USER $TXFLAG -y

# GET_AGENT_IDS='{"get_agent_ids":{}}'
# junod query wasm contract-state smart $CONTRACT "$GET_AGENT_IDS" $NODE