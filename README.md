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
# For building + fmt
./build.sh

# For testing everything
./test.sh

# For schemas
./schema.sh

# Production compilation, run before deploying to live network
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.7
```

## Testnet exapmles

There are some scripts for testnet, mainnet and local node
```bash
cd contracts/cw-croncat/scripts
```

To build and deploy to testnet:
```bash
./testnet_deploy.sh owner agent user
```
`owner`, `agent`, `user` are wallets that should have some ujunox.
In this script `owner` deploys optimized code to the chain, instantiates the contract, `user` creates a simple task, `agent` registers as an egent and executes this task.

Save the address of the created contract:
```bash
CRONCAT_ADDRESS=juno123somecontract
```

You can create a reccuring task and see the list of tasks:
```bash
./testnet_create_reccuring_task.sh $CRONCAT_ADDRESS user
./testnet_get_tasks.sh $CRONCAT_ADDRESS
```

For more exapmles for registering and unregistering agent, executing task and querying the state see other [scripts](https://github.com/CronCats/cw-croncat/tree/main/contracts/cw-croncat/scripts).

If you want to run croncat manager locally, see [instructions](https://github.com/CronCats/cw-croncat/blob/main/contracts/cw-croncat/scripts/README.md) for local setup.

## Agent

```bash
git clone git@github.com:CronCats/croncat-rs.git
cd croncat-rs
```

Create and store new agent address
```bash
cargo run -- generate-mnemonic --new-name new-agent
AGENT_ADDR=juno123agentaddress
```
Refill `new-agent` balance before using it, so that the agent has some ujunox for `register-agent` and `proxy-call`s.

Register an agent
```bash
cargo run -- register-agent
```

Start daemon
```bash
cargo run -- daemon --sender-name mike
```

Unregister the agent
```bash
cargo run -- unregister-agent --sender-name mike
```
> Default `new-name` and `sender-name` is `agent`.

For other commands see
```bash
cargo run -- help
``` 

## Changelog

### `0.0.1`

Initial setup
