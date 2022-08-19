#!/bin/bash
. ./testnet_init_vars.sh
USER=juno1pd43m659naajmn2chkt6tna0uud2ywyp5dm4h3
CONTRACT=juno12z4hh9r3j9aurjn6ppkgyjrkuu4ugrdectsh792w8feyj56dhlssvntdls

RECURRING='{"create_task":{"task":{"interval":{"Block":15},"boundary":null,"cw20_coins":[],"stop_on_fail":false,"actions":[{"bank":{"send":{"amount":[{"amount":"1","denom":"ujunox"}],"to_address":"juno1yhqft6d2msmzpugdjtawsgdlwvgq3samrm5wrw"}}},{"bank":{"send":{"amount":[{"amount":"1","denom":"ujunox"}],"to_address":"juno15w7hw4klzl9j2hk4vq7r3vuhz53h3mlzug9q6s"}}}],"rules":[]}}}'
junod tx wasm execute $CONTRACT "$RECURRING" --amount 500000ujunox --from $USER $TXFLAG -y

# GET_AGENT_IDS='{"get_agent_ids":{}}'
# junod query wasm contract-state smart $CONTRACT "$GET_AGENT_IDS" $NODE

