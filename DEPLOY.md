# Deploying the CronCat contracts

To ensure the optimization will work, it's a good idea to build the project first, which may update the `Cargo.lock` file, which is expected by the Docker optimizer.

    just build

Then optimize them with:

    just optimize

The contracts will appear in the `artifacts` directory. If you want to be certain that a contract was just updated, you can run `date` and then `ls -al artifacts` and compare the dates.

## Factory contract

The factory contract is in charge of keeping track of new CronCat releases. It uses cross-contract calls to store and instantiate the various contracts that make up CronCat. We'll use the factory to instantiate the agent contract here, and the same methodology applies to the other contracts.

To demonstrate the full flow, we start with the factory. The factory will be stored and instantiated.

**Note**: these commands use `stake` as the native token. Please adjust accordingly, like `ujunox` for Juno testnet, for instance.

Store factory with:

    CRONCAT_FACTORY_ID=$(junod tx wasm store artifacts/croncat_factory.wasm --from owner --gas-prices 0.025stake --gas auto --gas-adjustment 1.3 -b block -o json -y | jq -r ".logs[0].events[1].attributes[1].value")

The `code_id` is now stored in `CRONCAT_FACTORY_ID` and you can see it with `echo $CRONCAT_FACTORY_ID`.

Instantiate it with an owner address (`owner_addr`) if you wish to set an owner other than yourself. The command below doesn't include an explicit owner, so it'll use the `--from` address:

    CRONCAT_FACTORY_ADDR=$(junod tx wasm instantiate $CRONCAT_FACTORY_ID '{}' --from owner --gas-prices 0.025stake --gas auto --gas-adjustment 1.3 -b block -o json -y --admin $(junod keys show owner -aa) --label "CronCat-factory-alpha" | jq -r ".logs[0].events[0].attributes[0].value")

The `_contract_address` is now stored in `CRONCAT_FACTORY_ADDR`.

### Manager contract

Similarly, we'll need to know the `code_id` **and address** for the manager contract. We'll start by getting the ID:

    CRONCAT_MANAGER_ID=$(junod tx wasm store artifacts/croncat_manager.wasm --from owner --gas-prices 0.025stake --gas auto --gas-adjustment 1.3 -b block -o json -y | jq -r ".logs[0].events[1].attributes[1].value")

Then we'll get the manager's address. But we're going to use the factory to instantiate it!

We'll be sending a contract call to the factory's `deploy` method, but we'll need to make sure we've got the manager's instantiate message turned into base64 first.

Manager instantiate contract message:

```json
{
  "denom": "stake",
  "croncat_factory_addr": "", // We'll pass in CRONCAT_FACTORY_ADDR
  "croncat_tasks_key": [
    "t",
    [
      0,
      1
    ]
  ],
  "croncat_agents_key": [
    "a",
    [
      0,
      1
    ]
  ]
}
```

We can turn this into base64 with:

    CRONCAT_MANAGER_INST_MSG=$(echo '{"denom":"stake","croncat_factory_addr":"'$CRONCAT_FACTORY_ADDR'","croncat_tasks_key":["t",[0,1]],"croncat_agents_key":["a",[0,1]]}' | base64)

```json
{
  "deploy": {
    "kind": "manager",
    "module_instantiate_info": {
      "code_id": 0, // We'll pass in CRONCAT_MANAGER_ID
      "version": [
        0,
        1
      ],
      "commit_id": "8e08b808465c42235f961423fcf9e4792ce02462",
      "checksum": "abc123",
      "changelog_url": "https://example.com/lucky",
      "schema": "https://croncat-schema.example.com/version-0-1",
      "msg": "", // We'll pass in CRONCAT_MANAGER_INST_MSG
      "contract_name": "croncat-manager--version-0-1"
    }
  }
}
```

We'll minify the JSON and plug in the environment variables into a single line with:

    CRONCAT_FACTORY_DEPLOY_MANAGER=$(echo '{"deploy":{"kind":"manager","module_instantiate_info":{"code_id":'$CRONCAT_MANAGER_ID',"version":[0,1],"commit_id":"8e08b808465c42235f961423fcf9e4792ce02462","checksum":"abc123","changelog_url":"https://example.com/lucky","schema":"https://croncat-schema.example.com/version-0-1","msg":"'$CRONCAT_MANAGER_INST_MSG'","contract_name":"croncat-manager--version-0-1"}}}')

Finally, let's deploy the manager contract via the factory:

    CRONCAT_MANAGER_ADDR=$(junod tx wasm execute $CRONCAT_FACTORY_ADDR $CRONCAT_FACTORY_DEPLOY_MANAGER --from owner --gas-prices 0.025stake --gas auto --gas-adjustment 1.3 -b block -o json -y | jq -r ".logs[0].events[1].attributes[0].value")

## Agent contract

Next, we'll want to tell the factory to deploy our **agent** contract, but we'll need to pass it the `code_id` for it. We don't have this because we need to first store it:

    CRONCAT_AGENT_ID=$(junod tx wasm store artifacts/croncat_agents.wasm --from owner --gas-prices 0.025stake --gas auto --gas-adjustment 1.3 -b block -o json -y | jq -r ".logs[0].events[1].attributes[1].value")

Now let's call the factory, telling it to instantiate (and keep track of) a version of the agent contract.

One of the parameters will be a base64-encoded version of the instantiate message for the agents contract, so let's do that separately.

To instantiate the agent contract, we'll use this:

    CRONCAT_AGENT_INST_MSG=$(echo '{"manager_addr":"'$CRONCAT_MANAGER_ADDR'"}' | base64) 

**Note**: you may see the optional fields by looking for the `InstantiateMsg` in `packages/croncat-sdk-agents/src/msg.rs`

Back to the payload we're sending to the factory `deploy` method, let's take a look at the human-readable JSON:

```json
{
  "deploy": {
    "kind": "agents",
    "module_instantiate_info": {
      "code_id": 0, // We'll pass in CRONCAT_AGENT_ID
      "version": [
        0,
        1
      ],
      "commit_id": "8e08b808465c42235f961423fcf9e4792ce02462",
      "checksum": "abc123",
      "changelog_url": "https://example.com/lucky",
      "schema": "https://croncat-schema.example.com/version-0-1",
      "msg": "", // We'll pass in CRONCAT_AGENT_INST_MSG
      "contract_name": "croncat-agents--version-0-1"
    }
  }
}
```

Like we did with the manager, let's capture the parameters into an environment variable, this time for the agent's instantiation:

    CRONCAT_FACTORY_DEPLOY_AGENTS=$(echo '{"deploy":{"kind":"agents","module_instantiate_info":{"code_id":'$CRONCAT_AGENT_ID',"version":[0,1],"commit_id":"8e08b808465c42235f961423fcf9e4792ce02462","checksum":"abc123","changelog_url":"https://example.com/lucky","schema":"https://croncat-schema.example.com/version-0-1","msg":"'$CRONCAT_AGENT_INST_MSG'","contract_name":"croncat-agents--version-0-1"}}}')

We can minify (or reorganize to fit on one line) the JSON and get the parameters, then run the deploy command:

    CRONCAT_AGENT_ADDR=$(junod tx wasm execute $CRONCAT_FACTORY_ADDR $CRONCAT_FACTORY_DEPLOY_AGENTS --from owner --gas-prices 0.025stake --gas auto --gas-adjustment 1.3 -b block -o json -y | jq -r ".logs[0].events[1].attributes[0].value")
