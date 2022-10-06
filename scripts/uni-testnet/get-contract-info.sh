#!/bin/bash

cd "$(dirname "$0")"
. ./init-vars.sh

if [ -z "$1" ]
then
    echo "Must provide contract address"
    exit 1
else 
    CONTRACT=$1
fi

junod query wasm contract $CONTRACT $NODE
