#!/bin/bash
. ./testnet_init_vars.sh
AGENT=juno1pd43m659naajmn2chkt6tna0uud2ywyp5dm4h3
CONTRACT=juno12z4hh9r3j9aurjn6ppkgyjrkuu4ugrdectsh792w8feyj56dhlssvntdls
UNREGISTER_AGENT='{"unregister_agent":{}}'
junod tx wasm execute $CONTRACT "$UNREGISTER_AGENT" --from $AGENT $TXFLAG -y