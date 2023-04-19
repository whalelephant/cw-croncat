# CronCat Integration Utilities

This repo contains helper methods for projects adding automation to their smart contracts using CronCat tasks.

There are helper constants, functions, and [`cw-multi-test`](https://github.com/CosmWasm/cw-multi-test) exports that'll aid dApps in testing their custom workflows that leverage CronCat automation.

- `set_up_croncat_contracts` — you may provide an optional `cw-multi-test` `App` object, and it will set up the CronCat contracts and return this struct with helpful variables to be used in unit tests.

```rs
pub struct CronCatTestEnv {
    pub app: cw_multi_test::App,
    pub factory: cosmwasm_std::Addr,
    pub manager: cosmwasm_std::Addr,
    pub tasks: cosmwasm_std::Addr,
    pub agents: cosmwasm_std::Addr,
}
```

- `add_seconds_to_block` — convenience method to move time forward in the `cw-multi-test` environment
- `increment_block_height` — convenience method to increase block height in the `cw-multi-test` environment

There are additional exposed methods that will allow integrators to store and instantiate the various CronCat contracts.
