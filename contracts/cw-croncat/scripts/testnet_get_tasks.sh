. ./testnet_init_vars.sh
CONTRACT=juno12z4hh9r3j9aurjn6ppkgyjrkuu4ugrdectsh792w8feyj56dhlssvntdls
GET_TASKS='{"get_tasks":{}}'

junod query wasm contract-state smart $CONTRACT "$GET_TASKS" $NODE