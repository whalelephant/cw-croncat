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
npm i
```

## Next: Check your local Accounts

```bash
# Display accounts for specific network or all networks (see .env for supported networks list)
npm run accounts
npm run accounts junotestnet

# NOTE: Part of deployment requires a multi-sig contract for Pauser Admin.
# Be sure to create these separately in Mainnet, this scripting should only be used for testnet!!!!
```

## Make it go

```bash
# Deploys all the things, reporting contract addresses
# Deploys to a specific network or all networks (see .env for supported networks list)
npm run go
npm run go junotestnet

# you will see a pretty table printed if successful.
# Go to /artifacts and look for "chain_name-_deployed_contracts.json"

# runs full scope of contexts for end to end testing
# Runs on a specific network or all networks (see .env for supported networks list)
npm run e2e
npm run e2e junotestnet
```

## Add agent to address whitelist

```bash
# adds an agent address to the whitelist, so they can register
npm run whitelist stars15434j0vvv8un4hs0sfx8avmnc7yp...
```
