use std::{fmt::Display, str::FromStr};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    Addr, Api, Binary, CosmosMsg, Empty, Env, StdError, StdResult, Timestamp, Uint128, Uint64,
    WasmMsg,
};
use cron_schedule::Schedule;
use croncat_mod_generic::types::PathToValue;
pub use croncat_sdk_core::types::AmountForOneTask;
use cw20::{Cw20Coin, Cw20CoinVerified, Cw20ExecuteMsg};
use hex::ToHex;
use sha2::{Digest, Sha256};

#[cw_serde]
pub struct Config {
    // Runtime
    pub paused: bool,

    /// Address of the contract owner
    pub owner_addr: Addr,

    /// Address of the factory contract
    pub croncat_factory_addr: Addr,

    /// Chain name to add prefix to the task_hash
    pub chain_name: String,

    /// Name of the key for raw querying Manager address from the factory
    pub croncat_manager_key: (String, [u8; 2]),

    /// Name of the key for raw querying Agents address from the factory
    pub croncat_agents_key: (String, [u8; 2]),

    /// Time in nanos for each bucket of tasks
    pub slot_granularity_time: u64,

    /// Gas needed to cover proxy call without any action
    pub gas_base_fee: u64,

    /// Gas needed to cover single non-wasm task's Action
    pub gas_action_fee: u64,

    /// Gas needed to cover single query
    pub gas_query_fee: u64,

    /// Gas limit, to make sure task won't lock contract
    pub gas_limit: u64,
}

/// Request to create a task
#[cw_serde]
pub struct TaskRequest {
    pub interval: Interval,
    pub boundary: Option<Boundary>,
    pub stop_on_fail: bool,
    pub actions: Vec<Action>,
    pub queries: Option<Vec<CroncatQuery>>,
    pub transforms: Option<Vec<Transform>>,

    /// How much of cw20 coin is attached to this task
    pub cw20: Option<Cw20Coin>,
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

/// Start and end block or timestamp when task should be executed for the last time
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

impl From<BoundaryValidated> for Boundary {
    fn from(value: BoundaryValidated) -> Self {
        if value.is_block_boundary {
            Boundary::Height {
                start: Some(value.start.into()),
                end: value.end.map(Into::into),
            }
        } else {
            Boundary::Time {
                start: Some(Timestamp::from_nanos(value.start)),
                end: value.end.map(Timestamp::from_nanos),
            }
        }
    }
}

#[cw_serde]
pub struct Action<T = Empty> {
    /// Supported CosmosMsgs only!
    pub msg: CosmosMsg<T>,

    /// The gas needed to safely process the execute msg
    pub gas_limit: Option<u64>,
}

/// Transforms of the tasks actions
#[cw_serde]
pub struct Transform {
    /// Action index to update
    /// first action would be "0"
    pub action_idx: u64,

    /// Query index of the new data for this action
    /// first query would be "0"
    pub query_idx: u64,

    /// Action key path to the value that should get replaced
    /// for example:
    /// X: {Y: {Z: value}}
    /// [X,Y,Z] to reach that value
    pub action_path: PathToValue,
    /// Query response key's path to the value that needs to be taken to replace value from the above
    /// for example query gave that response:
    /// A: {B: {C: value}}
    /// In order to reach a value [A,B,C] should be used as input
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

    pub fn into_response(self, prefix: &str) -> TaskResponse {
        let task_hash = self.to_hash(prefix);
        let boundary = self.boundary.into();

        let queries = if !self.queries.is_empty() {
            Some(self.queries)
        } else {
            None
        };

        TaskResponse {
            task_hash,
            owner_addr: self.owner_addr,
            interval: self.interval,
            boundary,
            stop_on_fail: self.stop_on_fail,
            amount_for_one_task: self.amount_for_one_task,
            actions: self.actions,
            queries,
            transforms: self.transforms,
            version: self.version,
        }
    }

    /// Replace values to the result value from the rules
    /// Recalculate cw20 usage if any replacements
    pub fn replace_values(
        &mut self,
        api: &dyn Api,
        cron_addr: &Addr,
        construct_res_data: Vec<cosmwasm_std::Binary>,
    ) -> StdResult<()> {
        for transform in self.transforms.iter() {
            let wasm_msg = self
                .actions
                .get_mut(transform.action_idx as usize)
                .and_then(|action| {
                    if let CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: _,
                        msg,
                        funds: _,
                    }) = &mut action.msg
                    {
                        Some(msg)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| StdError::generic_err("Task is no longer valid"))?;
            let mut action_value = cosmwasm_std::from_binary(wasm_msg)?;

            let mut q_val = construct_res_data
                .get(transform.query_idx as usize)
                .ok_or_else(|| StdError::generic_err("Task is no longer valid"))
                .and_then(cosmwasm_std::from_binary)?;
            let replace_value = transform.query_response_path.find_value(&mut q_val)?;
            let replaced_value = transform.action_path.find_value(&mut action_value)?;
            *replaced_value = replace_value.clone();
            *wasm_msg = Binary(
                serde_json_wasm::to_vec(&action_value)
                    .map_err(|e| StdError::generic_err(e.to_string()))?,
            );
        }
        let cw20_amount_recalculated = self.recalculate_cw20_usage(api, cron_addr)?;
        self.amount_for_one_task.cw20 = cw20_amount_recalculated;
        Ok(())
    }

    fn recalculate_cw20_usage(
        &self,
        api: &dyn Api,
        cron_addr: &Addr,
    ) -> StdResult<Option<Cw20CoinVerified>> {
        let Some(current_cw20) = &self.amount_for_one_task.cw20 else {
            return Ok(None)
        };
        let actions = self.actions.iter();
        let mut cw20_amount = Uint128::zero();
        for action in actions {
            if let CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr, msg, ..
            }) = &action.msg
            {
                if cron_addr.as_str().eq(contract_addr) {
                    return Err(StdError::generic_err("Task is no longer valid"));
                }
                let validated_addr = api.addr_validate(contract_addr)?;
                if let Ok(cw20_msg) = cosmwasm_std::from_binary::<Cw20ExecuteMsg>(msg) {
                    // Don't let change type of cw20
                    if validated_addr != current_cw20.address {
                        return Err(StdError::generic_err("Task is no longer valid"));
                    }
                    match cw20_msg {
                        Cw20ExecuteMsg::Send { amount, .. } if !amount.is_zero() => {
                            cw20_amount = cw20_amount
                                .checked_add(amount)
                                .map_err(StdError::overflow)?;
                        }
                        Cw20ExecuteMsg::Transfer { amount, .. } if !amount.is_zero() => {
                            cw20_amount = cw20_amount
                                .checked_add(amount)
                                .map_err(StdError::overflow)?;
                        }
                        _ => {
                            return Err(StdError::generic_err("Task is no longer valid"));
                        }
                    }
                }
            }
        }
        Ok(Some(Cw20CoinVerified {
            address: current_cw20.address.clone(),
            amount: cw20_amount,
        }))
    }
}

/// Query given module contract with a message
#[cw_serde]
pub struct CroncatQuery {
    pub query_mod_addr: String,
    pub msg: Binary,
}

#[cw_serde]
pub struct SlotTasksTotalResponse {
    pub block_tasks: u64,
    pub cron_tasks: u64,
}

#[cw_serde]
pub struct TaskResponse {
    pub task_hash: String,

    pub owner_addr: Addr,

    pub interval: Interval,
    pub boundary: Boundary,

    pub stop_on_fail: bool,
    pub amount_for_one_task: AmountForOneTask,

    pub actions: Vec<Action>,
    pub queries: Option<Vec<CroncatQuery>>,
    pub transforms: Vec<Transform>,
    pub version: String,
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
