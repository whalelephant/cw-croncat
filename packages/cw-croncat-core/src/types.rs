use cosmwasm_std::{Addr, Binary, Coin, CosmosMsg, Timestamp};
use cw20::{Balance, Cw20CoinVerified};
use hex::encode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct GenericBalance {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20CoinVerified>,
}

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

/// Defines the spacing of execution
/// NOTE:S
/// - Block Height Based: Once, Immediate, Block
/// - Timestamp Based: Cron
/// - No Epoch support directly, advised to use block heights instead
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Interval {
    /// For when this is a non-recurring future scheduled TXN
    Once,

    /// The ugly batch schedule type, in case you need to exceed single TXN gas limits, within fewest block(s)
    Immediate,

    /// Allows timing based on block intervals rather than timestamps
    Block(u64),

    /// Crontab Spec String
    Cron(String),
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, JsonSchema)]
pub enum BoundarySpec {
    /// Represents the block height
    Height(u64),

    /// Represents the block timestamp
    Time(Timestamp),
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, JsonSchema)]
pub struct Boundary {
    ///
    pub start: Option<BoundarySpec>,
    ///
    pub end: Option<BoundarySpec>,
}

#[derive(Debug, PartialEq)]
pub enum SlotType {
    Block,
    Cron,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Rule {
    /// TBD: Interchain query support (See ibc::IbcMsg)
    pub chain_id: Option<String>,

    /// Account to direct all view calls against
    pub contract_id: Addr,

    // NOTE: Only allow static pre-defined query msg
    pub msg: Binary,
}

/// The response required by all rule queries. Bool is needed for croncat, T allows flexible rule engine
pub type RuleResponse<T> = (bool, T);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Task {
    /// Entity responsible for this task, can change task details
    pub owner_id: Addr,

    /// Scheduling definitions
    pub interval: Interval,
    pub boundary: Boundary,

    /// Defines if this task can continue until balance runs out
    pub stop_on_fail: bool,

    /// NOTE: Only tally native balance here, manager can maintain token/balances outside of tasks
    pub total_deposit: Vec<Coin>,

    /// The cosmos message to call, if time or rules are met
    pub action: CosmosMsg,
    // TODO: Decide if batch should be supported? Does that break gas limits ESP when rules are applied?
    // pub action: Vec<CosmosMsg>,
    /// A prioritized list of messages that can be chained decision matrix
    /// required to complete before task action
    /// Rules MUST return the ResolverResponse type
    pub rules: Option<Vec<Rule>>,
}

impl Task {
    /// Get the hash of a task based on parameters
    pub fn to_hash(&self) -> String {
        let message = format!(
            "{:?}{:?}{:?}{:?}{:?}",
            self.owner_id,
            self.interval,
            self.clone().boundary,
            self.action,
            self.rules
        );

        let hash = Sha256::digest(message.as_bytes());
        encode(hash)
    }
    /// Get the hash of a task based on parameters
    pub fn to_hash_vec(&self) -> Vec<u8> {
        self.to_hash().into_bytes()
    }
    // /// Returns the base amount required to execute 1 task
    // /// NOTE: this is not the final used amount, just the user-specified amount total needed
    // pub fn task_balance_uses(&self, task: &Task) -> u128 {
    //     task.deposit.0 + (u128::from(task.gas) * self.gas_price) + self.agent_fee
    // }
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
    pub fn minus_tokens(&mut self, minus: Balance) {
        match minus {
            Balance::Native(balance) => {
                for token in balance.0 {
                    let index = self.native.iter().enumerate().find_map(|(i, exist)| {
                        if exist.denom == token.denom {
                            Some(i)
                        } else {
                            None
                        }
                    });
                    if let Some(idx) = index {
                        self.native[idx].amount -= token.amount
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
                if let Some(idx) = index {
                    self.cw20[idx].amount -= token.amount
                }
            }
        };
    }
}
