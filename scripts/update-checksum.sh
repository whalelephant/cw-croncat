#!/bin/bash

cat artifacts/checksums.txt | grep -e cw_croncat.wasm -e cw_rules.wasm > checksum