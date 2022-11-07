# CronCat core

This library is based on CosmWasm and contains all messages, types and traits for the [CronCat Manager contract](https://github.com/CronCats/cw-croncat).
It is also used by [CronCat Agent](https://github.com/CronCats/croncat-rs) and [rules contract](https://github.com/CronCats/cw-croncat/tree/main/contracts/cw-rules).

## Messages

### Instantiate message

`InstantiateMsg` defines `denom`, `cw_rules_addr` for contract address for the rules and some other optional parameters. 

### Execute message

```rust
pub enum ExecuteMsg {
    // Task managment
    CreateTask {
        task: TaskRequest,
    },
    RemoveTask {
        task_hash: String,
    },
    RefillTaskBalance {
        task_hash: String,
    },
    RefillTaskCw20Balance {..},

    // For handling agent
    RegisterAgent {..},
    UpdateAgent {..},
    CheckInAgent {},
    UnregisterAgent {},
    WithdrawReward {},

    // Executing a task
    ProxyCall {
        task_hash: Option<String>,
    },

    // Updating config 
    UpdateSettings {..},

    // Moving balance to DAO or letting treasury transfer to itself
    MoveBalances {..},

    // Adding and withdrawing cw20 coins from user balance
    Receive(cw20::Cw20ReceiveMsg),
    WithdrawWalletBalance {
        cw20_amounts: Vec<Cw20Coin>,
    },

    // Helps manage and cleanup agents
    Tick {},
}
```

### Query messages

```rust
pub enum QueryMsg {
    // Query all tasks with rules starting with `from_index` till `limit`
    GetTasks {
        from_index: Option<u64>,
        limit: Option<u64>,
    },
    // Query all tasks without rules starting with `from_index` till `limit`
    GetTasksWithRules {
        from_index: Option<u64>,
        limit: Option<u64>,
    },
    // Query tasks created by owner_id
    GetTasksByOwner {
        owner_id: String,
    },
    // Query task by hash and vice versa
    GetTask {
        task_hash: String,
    },
    GetTaskHash {
        task: Box<Task>,
    },

    // Queries the information about all agents or the specific agent with address `account_id`
    GetAgent {
        account_id: String,
    },
    GetAgentIds {},
    GetAgentTasks {
        account_id: String,
    },

    // Checks if the interval is valid, returns bool
    ValidateInterval {
        interval: Interval,
    },

    // Query the current config
    GetConfig {},

    // Query native and cw20 balances
    GetBalances {},
    // Query user's cw20 balances
    GetWalletBalances {
        wallet: String,
    },

    // Query a list of active slot ids 
    GetSlotIds {},
    // Query list of task hashes by the slot
    GetSlotHashes {
        slot: Option<u64>,
    },

    // Returns the current state, including config, tasks and agents
    GetState {
        from_index: Option<u64>,
        limit: Option<u64>,
    },
}
```
