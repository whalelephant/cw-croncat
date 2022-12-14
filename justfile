test_addrs := env_var_or_default('TEST_ADDR', `jq -r '.[].address' ci/test_accounts.json | tr '\n' ' '`)
set export
check:
	cargo fmt && cargo clippy -- -D warnings
build:
	./scripts/build.sh
deploy:
	./scripts/uni-testnet/start.sh -c -w
checksum:
	./scripts/update-checksum.sh
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

optimize:
	docker run --rm -v "$(pwd)":/code \
		--mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		--platform linux/amd64 \
		cosmwasm/workspace-optimizer:0.12.8

download-deps:
	mkdir -p artifacts target
	wget https://github.com/CosmWasm/cw-plus/releases/latest/download/cw20_base.wasm -O artifacts/cw20_base.wasm
# TODO?: test dao-contracts

gas-benchmark: juno-local download-deps optimize
	#!/usr/bin/env bash
	sleep 1
	set -euxo pipefail
	TXFLAG="--chain-id testing --gas-prices 0.025ujunox --gas auto --gas-adjustment 1.3 --broadcast-mode block"
	docker cp 'artifacts/' cosmwasm:/artifacts
	RULES_ID=$(docker exec -i cosmwasm junod tx wasm store "/artifacts/cw_rules.wasm" -y --from validator $TXFLAG --output json | jq -r '.logs[0].events[-1].attributes[-1].value')
	CRONCAT_ID=$(docker exec -i cosmwasm junod tx wasm store "/artifacts/cw_croncat.wasm" -y --from validator $TXFLAG --output json | jq -r '.logs[0].events[-1].attributes[-1].value')
	CW20_ID=$(docker exec -i cosmwasm junod tx wasm store "/artifacts/cw20_base.wasm" -y --from validator $TXFLAG --output json | jq -r '.logs[0].events[-1].attributes[-1].value')
	CW20_ID=$CW20_ID CRONCAT_ID=$CRONCAT_ID RULES_ID=$RULES_ID VALIDATOR_ADDR=$(docker exec -i cosmwasm junod query staking validators --output json | jq -r '.validators[0].operator_address') RUST_LOG=info cargo run --bin gas-benchmark