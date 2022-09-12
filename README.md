<div align="center">
  <h1>
    Cron.cat CW Contracts
  </h1>
  <p>
  The "crontracts" for cosmwasm runtime of croncat service
  </p>
</div>

## ALPHA: 

#### This repo is in early develop stage - use at your own risk!

## Contributing

* [Developing](./Developing.md) how to run tests and develop code. Or go through the
[online tutorial](https://docs.cosmwasm.com/) to get a better feel
of how to develop.
* [Publishing](./Publishing.md) contains useful information on how to publish your contract
to the world, once you are ready to deploy it on a running blockchain.

## Commands

```bash
# For building + fmt + schema
./build.sh

# For testing everything
./test.sh

# Production compilation, run before deploying to live network
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.7
```

## Changelog

### `0.0.1`

Initial setup