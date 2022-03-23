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

## CI Support

One note is that the CI runs all `cargo` commands
with `--locked` to ensure it uses the exact same versions as you have locally. This also means
you must have an up-to-date `Cargo.lock` file, which is not auto-generated.
The first time you set up the project (or after adding any dep), you should ensure the
`Cargo.lock` file is updated, so the CI will test properly. This can be done simply by
running `cargo check` or `cargo unit-test`.

## Changelog

### `0.0.1`

Initial setup