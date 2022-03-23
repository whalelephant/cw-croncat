use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Item;

// Balance & Fee Definitions
pub const ONE_JUNO: u128 = 1_000_000_000;
// pub const BASE_BALANCE: Coin = ONE_JUNO * 5; // safety overhead
pub const GAS_BASE_PRICE: u32 = 100_000_000;
pub const GAS_BASE_FEE: u32 = 3_000_000_000;
// actual is: 13534954161128, higher in case treemap rebalance
pub const GAS_FOR_CALLBACK: u32 = 30_000_000;
// pub const AGENT_BASE_FEE: Coin = 500_000_000_000_000_000_000; // 0.0005 Ⓝ (2000 tasks = 1 Ⓝ)
pub const STAKE_BALANCE_MIN: u128 = 10 * ONE_JUNO;

// Boundary Definitions
pub const MAX_BLOCK_TS_RANGE: u64 = 1_000_000_000_000_000_000;
pub const SLOT_GRANULARITY: u64 = 60_000_000_000; // 60 seconds in nanos
pub const AGENT_EJECT_THRESHOLD: u128 = 600; // how many slots an agent can miss before being ejected. 10 * 60 = 1hr
pub const NANO: u64 = 1_000_000_000;

pub enum StorageKeys {
    Config,
    Tasks,
    Agents,
    Slots,
    AgentsActive,
    AgentsPending,
    Triggers,
    TaskOwners,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    // Runtime
    pub paused: bool,
    pub owner_id: Addr,
    // pub treasury_id: Option<Addr>,

    // // Agent management
    // // The ratio of tasks to agents, where index 0 is agents, index 1 is tasks
    // // Example: [1, 10]
    // // Explanation: For every 1 agent, 10 tasks per slot are available.
    // // NOTE: Caveat, when there are odd number of tasks or agents, the overflow will be available to first-come first-serve. This doesnt negate the possibility of a failed txn from race case choosing winner inside a block.
    // // NOTE: The overflow will be adjusted to be handled by sweeper in next implementation.
    // pub agent_task_ratio: [u64; 2],
    // pub agent_active_index: u64,
    // pub agents_eject_threshold: u128,

    // // Economics
    // pub available_balance: Coin, // tasks + rewards balances
    // pub staked_balance: Coin,
    // pub agent_fee: Coin,
    // pub gas_price: Coin,
    // pub proxy_callback_gas: u32,
    // pub slot_granularity: u64,
}

pub const STATE: Item<State> = Item::new("state");
pub const CONFIG: Item<Config> = Item::new("config");
