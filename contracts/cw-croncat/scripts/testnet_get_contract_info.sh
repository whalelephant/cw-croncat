#!/bin/bash
. ./testnet_init_vars.sh
CONTRACT=juno12z4hh9r3j9aurjn6ppkgyjrkuu4ugrdectsh792w8feyj56dhlssvntdls
junod query wasm contract $CONTRACT $NODE