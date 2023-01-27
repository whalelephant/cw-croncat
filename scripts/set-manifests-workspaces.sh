#!/bin/bash
#set -e

# Let's not worry about refactoring this until after the security audit

export RUSTFLAGS='-C link-arg=-s'
## Contracts
cd contracts || exit
### Agents
cd croncat-agents || exit
ln -sF manifests/workspaces-Cargo.toml Cargo.toml
### Factory
cd ../croncat-factory || exit
ln -sF manifests/workspaces-Cargo.toml Cargo.toml
## Manager
cd ../croncat-manager || exit
ln -sF manifests/workspaces-Cargo.toml Cargo.toml
## Tasks
cd ../croncat-tasks || exit
ln -sF manifests/workspaces-Cargo.toml Cargo.toml
## Mod Balances
cd ../mod-balances || exit
ln -sF manifests/workspaces-Cargo.toml Cargo.toml
## Mod DAO
cd ../mod-dao || exit
ln -sF manifests/workspaces-Cargo.toml Cargo.toml
## Mod Generic
cd ../mod-generic || exit
ln -sF manifests/workspaces-Cargo.toml Cargo.toml

## Packages
cd ../../packages || exit
## SDK Agents
cd croncat-sdk-agents || exit
ln -sF manifests/workspaces-Cargo.toml Cargo.toml
## SDK Core
cd ../croncat-sdk-core || exit
ln -sF manifests/workspaces-Cargo.toml Cargo.toml
## SDK Factory
cd ../croncat-sdk-factory || exit
ln -sF manifests/workspaces-Cargo.toml Cargo.toml
## SDK Manager
cd ../croncat-sdk-manager || exit
ln -sF manifests/workspaces-Cargo.toml Cargo.toml
## SDK Tasks
cd ../croncat-sdk-tasks || exit
ln -sF manifests/workspaces-Cargo.toml Cargo.toml
## Mod SDK
cd ../mod-sdk || exit
ln -sF manifests/workspaces-Cargo.toml Cargo.toml