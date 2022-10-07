#!/bin/bash
BINARY="docker exec -i juno-node-1 junod"

echo "Showing balances for:"
echo "validator"
$BINARY q bank balances $($BINARY keys show validator -a)
echo "owner"
$BINARY q bank balances $($BINARY keys show owner -a)
echo "agent"
$BINARY q bank balances $($BINARY keys show agent -a)
echo "user"
$BINARY q bank balances $($BINARY keys show user -a)
echo "alice"
$BINARY q bank balances $($BINARY keys show alice -a)
echo "bob"
$BINARY q bank balances $($BINARY keys show bob -a)
#Agent CW20 balance 
$BINARY query wasm contract-state smart $CW20_ADDR '{"balance": {"address": "'$AGENT_ADDR'"}}'