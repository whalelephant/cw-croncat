test_addrs := env_var_or_default('TEST_ADDR', `jq -r '.[].address' ci/test_accounts.json | tr '\n' ' '`)

check:
	cargo fmt && cargo clippy -- -D warnings

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
		ghcr.io/cosmoscontracts/juno:v10.0.2 /opt/setup_and_run.sh {{test_addrs}}

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

gas-benchmark: download-deps optimize juno-local
	sleep 1 # Need to sleep so validator is created
	VALIDATOR_ADDR=$(docker exec -i cosmwasm junod keys show validator -a) RUST_LOG=info cargo run --bin gas-benchmark