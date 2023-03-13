# Deploying CronCat contracts

Make sure you modify `.env` with the proper environment variables.

## Pre-requisites

```bash
# In root
just build
just optimize
```

## Make it so

```bash
cd scripts/deployment
yarn

# NOW!!! Go edit your `.env` with all the thangs
```

## Next: Check your local Accounts

```bash
# Display accounts for specific network or all networks (see .env for supported networks list)
yarn accounts
yarn accounts junotestnet

# NOTE: Part of deployment requires a multi-sig contract for Pauser Admin.
# Be sure to create these separately in Mainnet, this scripting should only be used for testnet!!!!
```

## Make it go

```bash
# Deploys all the things, reporting contract addresses
# Deploys to a specific network or all networks (see .env for supported networks list)
yarn go
yarn go junotestnet

# you will see a pretty table printed if successful.
# Go to /artifacts and look for "chain_name-_deployed_contracts.json"

# runs full scope of contexts for end to end testing
# Runs on a specific network or all networks (see .env for supported networks list)
yarn e2e
yarn e2e junotestnet
```

## Add agent to address whitelist

```bash
# adds an agent address to the whitelist, so they can register
yarn whitelist stars15434j0vvv8un4hs0sfx8avmnc7yp...
```

## Scenario coverage

```bash
# runs full scope of contexts for end to end testing
# NOTE: This WILL take quite a while, please allow for significant time for this to complete.
# Be advised to run only 1 network at a time, even tho it supports all
yarn e2eTasks junotestnet
```
