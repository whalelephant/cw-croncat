deploy:
	#!/bin/bash
	chmod +x ./scripts/testnet/deploy.sh
	./scripts/testnet/deploy.sh -w -c
optimize:
	docker run --rm -v "$(pwd)":/code \
		--mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		--platform linux/amd64 \
		cosmwasm/workspace-optimizer:0.12.10
