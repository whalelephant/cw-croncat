set export
lint:
	cargo fmt --all && cargo clippy -- -D warnings
test:
	#!/bin/bash
	cargo test -- --nocapture 

build:
	#!/bin/bash
	set -e
	export RUSTFLAGS='-C link-arg=-s'
	cargo build --release --lib --target wasm32-unknown-unknown
deploy:
	#!/bin/bash
	chmod +x ./scripts/testnet/deploy.sh
	./scripts/testnet/deploy.sh -w # only wasm update
deploy-local:
	#!/bin/bash
	chmod +x ./scripts/local/deploy.sh
	./scripts/local/deploy.sh -w # only wasm update

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
