use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::agent::Agent;
use crate::helpers::GenericBalance;
use crate::tasks::Task;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    // Runtime
    pub paused: bool,
    pub owner_id: Addr,

    // Agent management
    // The ratio of tasks to agents, where index 0 is agents, index 1 is tasks
    // Example: [1, 10]
    // Explanation: For every 1 agent, 10 tasks per slot are available.
    // NOTE: Caveat, when there are odd number of tasks or agents, the overflow will be available to first-come first-serve. This doesnt negate the possibility of a failed txn from race case choosing winner inside a block.
    // NOTE: The overflow will be adjusted to be handled by sweeper in next implementation.
    pub agent_task_ratio: [u64; 2],
    pub agent_active_index: u64,
    pub agents_eject_threshold: u64,

    // Economics
    pub agent_fee: Coin,
    pub gas_price: u32,
    pub proxy_callback_gas: u32,
    pub slot_granularity: u64,

    // Treasury
    // pub treasury_id: Option<Addr>,
    pub cw20_whitelist: Vec<Addr>, // TODO: Consider fee structure for whitelisted CW20s
    pub native_denom: String,
    pub available_balance: GenericBalance, // tasks + rewards balances
    pub staked_balance: GenericBalance, // surplus that is temporary staking (to be used in conjunction with external treasury)
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const AGENTS: Map<Addr, Agent> = Map::new("agents");
pub const AGENTS_ACTIVE_QUEUE: Item<Vec<Addr>> = Item::new("agent_active_queue");
pub const AGENTS_PENDING_QUEUE: Item<Vec<Addr>> = Item::new("agent_pending_queue");

// REF: https://github.com/CosmWasm/cw-plus/tree/main/packages/storage-plus#composite-keys
// Idea - create composite keys that are filterable to owners of tasks
pub const TASKS: Map<Vec<u8>, Task> = Map::new("tasks");
pub const TASK_OWNERS: Map<Addr, Vec<Vec<u8>>> = Map::new("task_owners");

// TODO: FINISH!!!!!!!!!!!
// TODO: Change this to an indexed / iterable key
/// Timestamps can be grouped into slot buckets (1-60 second buckets) for easier agent handling
pub const TIME_SLOTS: Map<u64, Vec<Vec<u8>>> = Map::new("time_slots");
/// Block slots allow for grouping of tasks at a specific block height,
/// this is done instead of forcing a block height into a range of timestamps for reliability
pub const BLOCK_SLOTS: Map<u64, Vec<Vec<u8>>> = Map::new("block_slots");

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{MockStorage};
//     use cosmwasm_std::{coins, BankMsg, CosmosMsg, StdResult};
//     use cw_storage_plus::{Bound, Order, Map};
//     use cw20::Balance;
//     use crate::slots::{Interval, Boundary};

//     pub const TASKS: Map<(Vec<u8>, Addr), Task> = Map::new("tasks");

//     #[test]
//     fn check_task_storage_structure() {
//         let mut store = MockStorage::new();

//         let to_address = String::from("you");
//         let amount = coins(1015, "earth");
//         let bank = BankMsg::Send { to_address, amount };
//         let msg: CosmosMsg = bank.clone().into();

//         let task = Task {
//             owner_id: Addr::unchecked("nobody".to_string()),
//             interval: Interval::Immediate,
//             boundary: Boundary {
//                 start: None,
//                 end: None,
//             },
//             stop_on_fail: false,
//             total_deposit: Balance::default(),
//             action: msg,
//             rules: None,
//         };

//         TASKS.save(&mut store, (task.to_hash_vec(), task.owner_id), &task);

//         // TODO: Test if i can do tasks + owners in same map with filtering
//     }
// }
