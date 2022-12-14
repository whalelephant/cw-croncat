#!/bin/bash

cat artifacts/checksums.txt | grep -e cw_croncat-aarch64.wasm -e cw_rules-aarch64.wasm > checksum
