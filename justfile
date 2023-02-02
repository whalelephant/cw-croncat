test_addrs := env_var_or_default('TEST_ADDR', `jq -r '.[].address' ci/test_accounts.json | tr '\n' ' '`)

set export
lint:
	cargo fmt --all && cargo clippy --all-targets -- -D warnings
test:
	#!/bin/bash
	cargo test -- --nocapture 
tarp:
	#!/bin/bash
	cargo +nightly tarpaulin --skip-clean --workspace --out Xml --target-dir tmp

build:
	#!/bin/bash
	set -e
	export RUSTFLAGS='-C link-arg=-s'
	cargo build --release --lib --target wasm32-unknown-unknown
deploy:
	#!/bin/bash
	cd ./scripts/deployment
	yarn go
deploy-local:
	#!/bin/bash
	chmod +x ./scripts/local/deploy.sh
	./scripts/local/deploy.sh -w # only wasm update

juno-local:
	docker kill cosmwasm || true
	docker volume rm -f junod_data
	docker run --rm -d --name cosmwasm \
		-e PASSWORD=xxxxxxxxx \
		-e STAKE_TOKEN=ujunox \
		-e GAS_LIMIT=100000000 \
		-e MAX_BYTES=22020096 \
		-e UNSAFE_CORS=true \
		-p 1317:1317 \
		-p 26656:26656 \
		-p 26657:26657 \
		-p 9090:9090 \
		--mount type=volume,source=junod_data,target=/root \
		ghcr.io/cosmoscontracts/juno:v11.0.3 /opt/setup_and_run.sh {{test_addrs}}

deploy-local-reset:
	#!/bin/bash
	chmod +x ./scripts/local/deploy.sh
	./scripts/local/deploy.sh -w  -c #  wasm update & container update

optimize:
	docker run --rm -v "$(pwd)":/code \
		--mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		--platform linux/amd64 \
		cosmwasm/workspace-optimizer:0.12.10

gen-schema:
	./scripts/schema.sh

gen-typescript:
	yarn --cwd ./typescript install --frozen-lockfile
	yarn --cwd ./typescript build
	yarn --cwd ./typescript codegen

checksum:
	#!/bin/bash
	cat artifacts/checksums.txt | grep -e croncat_agents.wasm -e croncat_factory.wasm -e croncat_manager.wasm -e croncat_mod_balances.wasm -e croncat_mod_dao.wasm -e croncat_mod_generic.wasm -e croncat_mod_nft.wasm -e croncat_tasks.wasm > checksum

schema: gen-schema gen-typescript

all: lint build schema test checksum