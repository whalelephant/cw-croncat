#!/bin/bash

# Note: the below command has amd64 as the target, meaning M1 macs
# Please use these directions if you're not on one:
#   https://docs.cosmwasm.com/docs/1.0/getting-started/compile-contract#optimized-compilation
  docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  --platform linux/arm64 \
  cosmwasm/workspace-optimizer:0.12.6
