use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin};
use cw20::{Balance, Cw20CoinVerified};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct GenericBalance {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20CoinVerified>,
}

impl GenericBalance {
    pub fn add_tokens(&mut self, add: Balance) {
        match add {
            Balance::Native(balance) => {
                for token in balance.0 {
                    let index = self.native.iter().enumerate().find_map(|(i, exist)| {
                        if exist.denom == token.denom {
                            Some(i)
                        } else {
                            None
                        }
                    });
                    match index {
                        Some(idx) => self.native[idx].amount += token.amount,
                        None => self.native.push(token),
                    }
                }
            }
            Balance::Cw20(token) => {
                let index = self.cw20.iter().enumerate().find_map(|(i, exist)| {
                    if exist.address == token.address {
                        Some(i)
                    } else {
                        None
                    }
                });
                match index {
                    Some(idx) => self.cw20[idx].amount += token.amount,
                    None => self.cw20.push(token),
                }
            }
        };
    }
}

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
    pub agents_eject_threshold: u128,

    // Economics
    pub agent_fee: Coin,
    pub gas_price: u32,
    pub proxy_callback_gas: u32,
    pub slot_granularity: u64,

    // Treasury
    pub treasury_id: Option<Addr>,
    pub cw20_whitelist: Vec<Addr>, // TODO: Consider fee structure for whitelisted CW20s
    pub available_balance: GenericBalance, // tasks + rewards balances
    pub staked_balance: GenericBalance, // surplus that is temporary staking (to be used in conjunction with external treasury)
}

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum AgentStatus {
    // Default for any new agent, if tasks ratio allows
    Active,

    // Default for any new agent, until more tasks come online
    Pending,

    // More tasks are available, agent must checkin to become active
    Nominated,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Agent {
    pub status: AgentStatus,

    // Where rewards get transferred
    pub payable_account_id: Addr,

    // accrued reward balance
    pub balance: GenericBalance,

    // stats
    pub total_tasks_executed: u64,

    // Holds slot number of a missed slot.
    // If other agents see an agent miss a slot, they store the missed slot number.
    // If agent does a task later, this number is reset to zero.
    // Example data: 1633890060000000000 or 0
    pub last_missed_slot: u64,

    // Timestamp of when agent first registered
    // Useful for rewarding agents for their patience while they are pending and operating service
    // Agent will be responsible to constantly monitor when it is their turn to join in active agent set (done as part of agent code loops)
    // Example data: 1633890060000000000 or 0
    pub register_start: u64,
}

pub const AGENTS: Map<Addr, Agent> = Map::new("agents");
pub const AGENTS_ACTIVE_QUEUE: Item<Vec<Addr>> = Item::new("agent_active_queue");
pub const AGENTS_PENDING_QUEUE: Item<Vec<Addr>> = Item::new("agent_pending_queue");
