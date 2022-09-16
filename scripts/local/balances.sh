#!/bin/bash

echo "Showing balances for:"
echo "validator"
junod q bank balances $(junod keys show validator -a)
echo "owner"
junod q bank balances $(junod keys show owner -a)
echo "agent"
junod q bank balances $(junod keys show agent -a)
echo "user"
junod q bank balances $(junod keys show user -a)
echo "alice"
junod q bank balances $(junod keys show alice -a)
echo "bob"
junod q bank balances $(junod keys show bob -a)