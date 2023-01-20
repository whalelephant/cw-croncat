use std::{fmt::Display, str::FromStr};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coin, Addr, Binary, Coin, CosmosMsg, Empty, Env, Timestamp, Uint128, Uint64};
use cron_schedule::Schedule;
use croncat_mod_generic::types::PathToValue;
use cw20::{Cw20Coin, Cw20CoinVerified};
use hex::ToHex;
use sha2::{Digest, Sha256};

#[cw_serde]
pub struct Config {
    // Runtime
    pub paused: bool,
    pub owner_addr: Addr,

    pub croncat_factory_addr: Addr,
    pub croncat_manager_key: (String, [u8; 2]),
    pub croncat_agents_key: (String, [u8; 2]),

    pub slot_granularity_time: u64,

    pub gas_base_fee: u64,
    pub gas_action_fee: u64,
    pub gas_query_fee: u64,

    pub gas_limit: u64,
}

#[cw_serde]
pub struct TaskRequest {
    pub interval: Interval,
    pub boundary: Option<Boundary>,
    pub stop_on_fail: bool,
    pub actions: Vec<Action>,
    pub queries: Option<Vec<CroncatQuery>>,
    pub transforms: Option<Vec<Transform>>,

    /// How much coins of this chain attach
    pub native: Uint128,
    /// How much of cw20 coin is attached to this task
    pub cw20: Option<Cw20Coin>,
    /// How much of native coin is attached to this task
    pub ibc: Option<Coin>,
}

/// Defines the spacing of execution
/// NOTES:
/// - Block Height Based: Once, Immediate, Block
/// - Timestamp Based: Once, Cron
/// - No Epoch support directly, advised to use block heights instead
#[cw_serde]
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

impl Interval {
    pub fn next(
        &self,
        env: &Env,
        boundary: &BoundaryValidated,
        slot_granularity_time: u64,
    ) -> (u64, SlotType) {
        match self {
            // If Once, return the first block within a specific range that can be triggered 1 time.
            // If Immediate, return the first block within a specific range that can be triggered immediately, potentially multiple times.
            Interval::Once | Interval::Immediate => {
                if boundary.is_block_boundary {
                    get_next_block_limited(env, boundary)
                } else {
                    get_next_cron_time(env, boundary, "0 0 * * * *", slot_granularity_time)
                }
            }
            // return the first block within a specific range that can be triggered 1 or more times based on timestamps.
            // Uses crontab spec
            Interval::Cron(crontab) => {
                get_next_cron_time(env, boundary, crontab, slot_granularity_time)
            }
            // return the block within a specific range that can be triggered 1 or more times based on block heights.
            // Uses block offset (Example: Block(100) will trigger every 100 blocks)
            // So either:
            // - Boundary specifies a start/end that block offsets can be computed from
            // - Block offset will truncate to specific modulo offsets
            Interval::Block(block) => get_next_block_by_offset(env, boundary, *block),
        }
    }

    pub fn is_valid(&self) -> bool {
        match self {
            Interval::Once | Interval::Immediate | Interval::Block(_) => true,
            Interval::Cron(crontab) => {
                let s = Schedule::from_str(crontab);
                s.is_ok()
            }
        }
    }
}

#[cw_serde]
pub enum Boundary {
    Height {
        start: Option<Uint64>,
        end: Option<Uint64>,
    },
    Time {
        start: Option<Timestamp>,
        end: Option<Timestamp>,
    },
}

#[cw_serde]
pub struct BoundaryValidated {
    pub start: u64,
    pub end: Option<u64>,
    pub is_block_boundary: bool,
}

#[cw_serde]
pub struct Action<T = Empty> {
    // NOTE: Only allow static pre-defined query msg
    /// Supported CosmosMsgs only!
    pub msg: CosmosMsg<T>,

    /// The gas needed to safely process the execute msg
    pub gas_limit: Option<u64>,
}

#[cw_serde]
pub struct Transform {
    pub action_idx: u64,
    pub query_idx: u64,

    pub action_path: PathToValue,
    pub query_response_path: PathToValue,
}

#[cw_serde]
pub struct Task {
    /// Entity responsible for this task, can change task details
    pub owner_addr: Addr,

    /// Scheduling definitions
    pub interval: Interval,
    pub boundary: BoundaryValidated,

    /// Defines if this task can continue until balance runs out
    pub stop_on_fail: bool,

    pub amount_for_one_task: AmountForOneTask,

    /// The cosmos message to call, if time or rules are met
    pub actions: Vec<Action>,
    /// A prioritized list of messages that can be chained decision matrix
    /// required to complete before task action
    /// Rules MUST return the ResolverResponse type
    pub queries: Vec<CroncatQuery>,
    pub transforms: Vec<Transform>,
    pub version: String,
}

impl Task {
    /// Get the hash of a task based on parameters
    pub fn to_hash(&self, prefix: &str) -> String {
        let message = format!(
            "{:?}{:?}{:?}{:?}{:?}{:?}",
            self.owner_addr,
            self.interval,
            self.boundary,
            self.actions,
            self.queries,
            self.transforms
        );

        let hash = Sha256::digest(message.as_bytes());
        let encoded: String = hash.encode_hex();

        // Return prefixed hash, since multi-chain tasks require simpler identification
        // Using the specified native_denom, if none, no prefix
        // Example:
        // No prefix:   fca49b82eb84818215768293c9e57e7d4194a7c862538e1dedb4516bf2dff0ca (No longer used/stored)
        // with prefix: stars:82eb84818215768293c9e57e7d4194a7c862538e1dedb4516bf2dff0ca
        // with prefix: longnetwork:818215768293c9e57e7d4194a7c862538e1dedb4516bf2dff0ca
        let (_, l) = encoded.split_at(prefix.len() + 1);
        format!("{}:{}", prefix, l)
    }

    /// Get the hash of a task based on parameters
    pub fn to_hash_vec(&self, prefix: &str) -> Vec<u8> {
        self.to_hash(prefix).into_bytes()
    }

    pub fn recurring(&self) -> bool {
        !matches!(self.interval, Interval::Once)
    }

    pub fn with_queries(&self) -> bool {
        !self.queries.is_empty()
    }
}

#[cw_serde]
pub struct CroncatQuery {
    pub query_mod: String,
    pub msg: Binary,
}

#[cw_serde]
#[derive(Default)]
pub struct AmountForOneTask {
    pub gas: u64,
    pub cw20: Option<Cw20CoinVerified>,
    pub coin: Option<Coin>,
}

impl AmountForOneTask {
    #[must_use]
    pub fn add_gas(&mut self, gas: u64, limit: u64) -> bool {
        self.gas = self.gas.saturating_add(gas);

        self.gas <= limit
    }

    #[must_use]
    pub fn add_coin(&mut self, coin: Coin) -> bool {
        if let Some(coin_inner) = &mut self.coin {
            if coin_inner.denom != coin.denom {
                return false;
            }
            coin_inner.amount += coin.amount;
        } else {
            self.coin = Some(coin);
        }
        true
    }

    #[must_use]
    pub fn add_cw20(&mut self, cw20: Cw20CoinVerified) -> bool {
        if let Some(cw20_inner) = &mut self.cw20 {
            if cw20_inner.address != cw20.address {
                return false;
            }
            cw20_inner.amount += cw20.amount;
        } else {
            self.cw20 = Some(cw20);
        }
        true
    }
}

#[cw_serde]
pub struct TaskResponse {
    pub task_hash: String,

    pub owner_id: Addr,

    pub interval: Interval,
    pub boundary: Option<Boundary>,

    pub stop_on_fail: bool,
    pub amount_for_one_task: AmountForOneTask,

    pub actions: Vec<Action>,
    // TODO: pub queries: Option<Vec<CroncatQuery>>,
}

#[cw_serde]
pub struct SlotHashesResponse {
    pub block_id: u64,
    pub block_task_hash: Vec<String>,
    pub time_id: u64,
    pub time_task_hash: Vec<String>,
}

#[cw_serde]
pub struct SlotIdsResponse {
    pub time_ids: Vec<u64>,
    pub block_ids: Vec<u64>,
}

pub enum SlotType {
    Block,
    Cron,
}

impl Display for SlotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlotType::Block => write!(f, "block"),
            SlotType::Cron => write!(f, "cron"),
        }
    }
}

/// Get the next block within the boundary
fn get_next_block_limited(env: &Env, boundary: &BoundaryValidated) -> (u64, SlotType) {
    let current_block_height = env.block.height;

    let next_block_height = if current_block_height < boundary.start {
        boundary.start - 1
    } else {
        current_block_height
    };

    match boundary.end {
        // stop if passed end height
        Some(end) if current_block_height > end => (0, SlotType::Block),

        // we ONLY want to catch if we're passed the end block height
        Some(end) if next_block_height > end => (end, SlotType::Block),
        // immediate needs to return this block + 1
        _ => (next_block_height + 1, SlotType::Block),
    }
}

/// Either:
/// - Boundary specifies a start/end that block offsets can be computed from
/// - Block offset will truncate to specific modulo offsets
fn get_next_block_by_offset(
    env: &Env,
    boundary: &BoundaryValidated,
    block: u64,
) -> (u64, SlotType) {
    let current_block_height = env.block.height;
    let modulo_block = current_block_height.saturating_sub(current_block_height % block) + block;

    let next_block_height = if current_block_height < boundary.start {
        let rem = boundary.start % block;
        if rem > 0 {
            boundary.start.saturating_sub(rem) + block
        } else {
            boundary.start
        }
    } else {
        modulo_block
    };

    match boundary.end {
        // stop if passed end height
        Some(end) if current_block_height > end => (0, SlotType::Block),

        // we ONLY want to catch if we're passed the end block height
        Some(end) => {
            let end_height = if let Some(rem) = end.checked_rem(block) {
                end.saturating_sub(rem)
            } else {
                end
            };
            (end_height, SlotType::Block)
        }

        None => (next_block_height, SlotType::Block),
    }
}

/// Get the slot number (in nanos) of the next task according to boundaries
/// Unless current slot is the end slot, don't put in the current slot
fn get_next_cron_time(
    env: &Env,
    boundary: &BoundaryValidated,
    crontab: &str,
    slot_granularity_time: u64,
) -> (u64, SlotType) {
    let current_block_ts = env.block.time.nanos();
    let current_block_slot =
        current_block_ts.saturating_sub(current_block_ts % slot_granularity_time);

    // get earliest possible time
    let current_ts = if current_block_ts < boundary.start {
        boundary.start
    } else {
        current_block_ts
    };

    // receive time from schedule, calculate slot for this time
    let schedule = Schedule::from_str(crontab).unwrap();
    let next_ts = schedule.next_after(&current_ts).unwrap();
    let next_ts_slot = next_ts.saturating_sub(next_ts % slot_granularity_time);

    // put task in the next slot if next_ts_slot in the current slot
    let next_slot = if next_ts_slot == current_block_slot {
        next_ts_slot + slot_granularity_time
    } else {
        next_ts_slot
    };

    match boundary.end {
        Some(end) if current_block_ts > end => (0, SlotType::Cron),
        Some(end) => {
            let end_slot = end.saturating_sub(end % slot_granularity_time);
            (u64::min(end_slot, next_slot), SlotType::Cron)
        }
        _ => (next_slot, SlotType::Cron),
    }
}
