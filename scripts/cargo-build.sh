#!/bin/bash
#set -e

# Let's not worry about refactoring this until after the security audit

export RUSTFLAGS='-C link-arg=-s'
## Contracts
cd contracts || exit
### Agents
cd croncat-agents || exit
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/*.wasm ../../artifacts/cargo-builds
### Factory
cd ../croncat-factory || exit
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/*.wasm ../../artifacts/cargo-builds
## Manager
cd ../croncat-manager || exit
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/*.wasm ../../artifacts/cargo-builds
## Tasks
cd ../croncat-tasks || exit
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/*.wasm ../../artifacts/cargo-builds
## Mod Balances
cd ../mod-balances || exit
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/*.wasm ../../artifacts/cargo-builds
## Mod DAO
cd ../mod-dao || exit
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/*.wasm ../../artifacts/cargo-builds
## Mod Generic
cd ../mod-generic || exit
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/*.wasm ../../artifacts/cargo-builds
## Mod NFT
cd ../mod-nft || exit
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/*.wasm ../../artifacts/cargo-builds

### Packages
#cd ../../packages || exit
### SDK Agents
#cd croncat-sdk-agents || exit
#cargo build --release --target wasm32-unknown-unknown
#cp target/wasm32-unknown-unknown/release/*.wasm ../../artifacts/cargo-builds
### SDK Core
#cd ../croncat-sdk-core || exit
#cargo build --release --target wasm32-unknown-unknown
#cp target/wasm32-unknown-unknown/release/*.wasm ../../artifacts/cargo-builds
### SDK Factory
#cd ../croncat-sdk-factory || exit
#cargo build --release --target wasm32-unknown-unknown
#cp target/wasm32-unknown-unknown/release/*.wasm ../../artifacts/cargo-builds
### SDK Manager
#cd ../croncat-sdk-manager || exit
#cargo build --release --target wasm32-unknown-unknown
#cp target/wasm32-unknown-unknown/release/*.wasm ../../artifacts/cargo-builds
### SDK Tasks
#cd ../croncat-sdk-tasks || exit
#cargo build --release --target wasm32-unknown-unknown
#cp target/wasm32-unknown-unknown/release/*.wasm ../../artifacts/cargo-builds
### Mod SDK
#cd ../mod-sdk || exit
#cargo build --release --target wasm32-unknown-unknown
#cp target/wasm32-unknown-unknown/release/*.wasm ../../artifacts/cargo-builds