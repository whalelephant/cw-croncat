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
```

## Make it go

```bash
# runs full scope of contexts for end to end testing
npm run e2e
```
