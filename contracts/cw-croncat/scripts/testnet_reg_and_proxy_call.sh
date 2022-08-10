. ./testnet_init_vars.sh

AGENT=juno1pd43m659naajmn2chkt6tna0uud2ywyp5dm4h3
CONTRACT=juno12z4hh9r3j9aurjn6ppkgyjrkuu4ugrdectsh792w8feyj56dhlssvntdls

REGISTER_AGENT='{"register_agent":{}}'
junod tx wasm execute $CONTRACT "$REGISTER_AGENT" --from $AGENT $TXFLAG -y

#make agent active
CHECK_IN_AGENT='{"check_in_agent":{}}'
junod tx wasm execute $CONTRACT "$CHECK_IN_AGENT" --from $AGENT $TXFLAG -y

PROXY_CALL='{"proxy_call":{}}'
junod tx wasm execute $CONTRACT "$PROXY_CALL" --from $AGENT $TXFLAG -y

echo "AGENT - " $AGENT
echo "CONTRACT - " $CONTRACT
