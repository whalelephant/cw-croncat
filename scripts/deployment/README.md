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
# Deploys all the things, reporting contract addresses
npm run go

# you will see a pretty table printed if successful.
# Go to /artifacts and look for "chain-id_deployed_contracts.json"
```

## Make it go

```bash
# runs full scope of contexts for end to end testing
npm run e2e
```

## Add agent to address whitelist

```bash
# adds an agent address to the whitelist, so they can register
npm run whitelist stars15434j0vvv8un4hs0sfx8avmnc7yp...
```
