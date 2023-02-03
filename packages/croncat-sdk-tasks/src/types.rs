use std::{fmt::Display, str::FromStr};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, CosmosMsg, Empty, Env, Timestamp, Uint64};
use cron_schedule::Schedule;
use croncat_mod_generic::types::PathToValue;
pub use croncat_sdk_core::types::AmountForOneTask;
use cw20::Cw20Coin;
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

    /// Assigned by Factory, denotes the version of this contract (CW2 spec) & used as the task verion as well.
    pub version: String,

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
    /// This will be taken from the manager's contract temporary "Users balance"
    /// and attached directly to the task's balance.
    ///
    /// Note: Unlike other coins ( which get refunded to the task creator in the same transaction as task removal)
    /// cw20's will get moved back to the temporary "Users balance".
    /// This is done primarily to save up gas from executing another contract during `proxy_call`
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
            Interval::Block(block) => get_next_block_by_offset(env.block.height, boundary, *block),
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
            task: Some(TaskInfo {
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
            }),
        }
    }
}

/// Query given module contract with a message
#[cw_serde]
pub struct CroncatQuery {
    /// This is address of the queried module contract.
    /// For the addr can use one of our croncat-mod-* contracts, or custom contracts
    ///
    /// One requirement for custom contracts: query return value should be formatted as a:
    /// [`QueryResponse`](mod_sdk::types::QueryResponse)
    pub contract_addr: String,
    pub msg: Binary,
    pub check_result: bool,
}

#[cw_serde]
pub struct SlotTasksTotalResponse {
    pub block_tasks: u64,
    pub cron_tasks: u64,
}

#[cw_serde]
pub struct TaskInfo {
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
pub struct TaskResponse {
    pub task: Option<TaskInfo>,
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

#[cw_serde]
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
pub(crate) fn get_next_block_by_offset(
    block_height: u64,
    boundary: &BoundaryValidated,
    block: u64,
) -> (u64, SlotType) {
    let current_block_height = block_height;
    let modulo_block = if block > 0 {
        current_block_height.saturating_sub(current_block_height % block) + block
    } else {
        return (0, SlotType::Block);
    };

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
            // we ONLY want to catch if we're passed the end block height
            (
                std::cmp::min(next_block_height, end_height),
                SlotType::Block,
            )
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

#[cfg(test)]
mod test {
    use cosmwasm_std::{testing::mock_env, Addr, CosmosMsg, WasmMsg};
    use croncat_sdk_core::types::AmountForOneTask;
    use hex::ToHex;
    use sha2::{Digest, Sha256};

    use crate::types::{Action, CroncatQuery, Transform};

    use super::{BoundaryValidated, Interval, SlotType, Task};

    const TWO_MINUTES: u64 = 120_000_000_000;

    #[test]
    fn is_valid_test() {
        let once = Interval::Once;
        assert!(once.is_valid());

        let immediate = Interval::Immediate;
        assert!(immediate.is_valid());

        let block = Interval::Block(100);
        assert!(block.is_valid());

        let cron_correct = Interval::Cron("1 * * * * *".to_string());
        assert!(cron_correct.is_valid());

        let cron_wrong = Interval::Cron("1 * * * * * *".to_string());
        assert!(cron_wrong.is_valid());
    }

    #[test]
    fn hashing() {
        let task = Task {
            owner_addr: Addr::unchecked("bob"),
            interval: Interval::Block(5),
            boundary: BoundaryValidated {
                start: 4,
                end: None,
                is_block_boundary: true,
            },
            stop_on_fail: false,
            amount_for_one_task: AmountForOneTask {
                gas: 100,
                cw20: None,
                coin: [None, None],
            },
            actions: vec![Action {
                msg: CosmosMsg::Wasm(WasmMsg::ClearAdmin {
                    contract_addr: "alice".to_string(),
                }),
                gas_limit: Some(5),
            }],
            queries: vec![CroncatQuery {
                msg: Default::default(),
                contract_addr: "addr".to_owned(),
                check_result: true,
            }],
            transforms: vec![Transform {
                action_idx: 0,
                query_idx: 0,
                action_path: vec![].into(),
                query_response_path: vec![].into(),
            }],
            version: String::from(""),
        };

        let message = format!(
            "{:?}{:?}{:?}{:?}{:?}{:?}",
            task.owner_addr,
            task.interval,
            task.boundary,
            task.actions,
            task.queries,
            task.transforms
        );

        let hash = Sha256::digest(message.as_bytes());

        let encode: String = hash.encode_hex();
        let prefix = "atom";
        let (_, l) = encode.split_at(prefix.len() + 1);
        let encoded = format!("{}:{}", prefix, l);
        let bytes = encoded.clone().into_bytes();

        // Tests
        assert_eq!(encoded, task.to_hash(prefix));
        assert_eq!(bytes, task.to_hash_vec(prefix));
    }

    #[test]
    fn interval_get_next_block_limited() {
        // (input, input, outcome, outcome)
        let cases: Vec<(Interval, BoundaryValidated, u64, SlotType)> = vec![
            // Once cases
            (
                Interval::Once,
                BoundaryValidated {
                    start: 12345,
                    end: None,
                    is_block_boundary: true,
                },
                12346,
                SlotType::Block,
            ),
            (
                Interval::Once,
                BoundaryValidated {
                    start: 12348,
                    end: None,
                    is_block_boundary: true,
                },
                12348,
                SlotType::Block,
            ),
            (
                Interval::Once,
                BoundaryValidated {
                    start: 12345,
                    end: Some(12346),
                    is_block_boundary: true,
                },
                12346,
                SlotType::Block,
            ),
            (
                Interval::Once,
                BoundaryValidated {
                    start: 12345,
                    end: Some(12340),
                    is_block_boundary: true,
                },
                0,
                SlotType::Block,
            ),
            // Immediate cases
            (
                Interval::Immediate,
                BoundaryValidated {
                    start: 12345,
                    end: None,
                    is_block_boundary: true,
                },
                12346,
                SlotType::Block,
            ),
            (
                Interval::Immediate,
                BoundaryValidated {
                    start: 12348,
                    end: None,
                    is_block_boundary: true,
                },
                12348,
                SlotType::Block,
            ),
            (
                Interval::Immediate,
                BoundaryValidated {
                    start: 12345,
                    end: Some(12346),
                    is_block_boundary: true,
                },
                12346,
                SlotType::Block,
            ),
            (
                Interval::Immediate,
                BoundaryValidated {
                    start: 12345,
                    end: Some(12340),
                    is_block_boundary: true,
                },
                0,
                SlotType::Block,
            ),
        ];
        // Check all these cases
        for (interval, boundary, outcome_block, outcome_slot_kind) in cases.iter() {
            let env = mock_env();
            let (next_id, slot_kind) = interval.next(&env, boundary, 1);
            assert_eq!(outcome_block, &next_id);
            assert_eq!(outcome_slot_kind, &slot_kind);
        }
    }

    #[test]
    fn interval_get_next_block_by_offset() {
        // (input, input, outcome, outcome)
        let cases: Vec<(Interval, BoundaryValidated, u64, SlotType)> = vec![
            // strictly modulo cases
            (
                Interval::Block(1),
                BoundaryValidated {
                    start: 12345,
                    end: None,
                    is_block_boundary: true,
                },
                12346,
                SlotType::Block,
            ),
            (
                Interval::Block(10),
                BoundaryValidated {
                    start: 12345,
                    end: None,
                    is_block_boundary: true,
                },
                12350,
                SlotType::Block,
            ),
            (
                Interval::Block(100),
                BoundaryValidated {
                    start: 12345,
                    end: None,
                    is_block_boundary: true,
                },
                12400,
                SlotType::Block,
            ),
            (
                Interval::Block(1000),
                BoundaryValidated {
                    start: 12345,
                    end: None,
                    is_block_boundary: true,
                },
                13000,
                SlotType::Block,
            ),
            (
                Interval::Block(10000),
                BoundaryValidated {
                    start: 12345,
                    end: None,
                    is_block_boundary: true,
                },
                20000,
                SlotType::Block,
            ),
            (
                Interval::Block(100000),
                BoundaryValidated {
                    start: 12345,
                    end: None,
                    is_block_boundary: true,
                },
                100000,
                SlotType::Block,
            ),
            // with another start
            (
                Interval::Block(1),
                BoundaryValidated {
                    start: 12348,
                    end: None,
                    is_block_boundary: true,
                },
                12348,
                SlotType::Block,
            ),
            (
                Interval::Block(10),
                BoundaryValidated {
                    start: 12360,
                    end: None,
                    is_block_boundary: true,
                },
                12360,
                SlotType::Block,
            ),
            (
                Interval::Block(10),
                BoundaryValidated {
                    start: 12364,
                    end: None,
                    is_block_boundary: true,
                },
                12370,
                SlotType::Block,
            ),
            (
                Interval::Block(100),
                BoundaryValidated {
                    start: 12364,
                    end: None,
                    is_block_boundary: true,
                },
                12400,
                SlotType::Block,
            ),
            // modulo + boundary end
            (
                Interval::Block(1),
                BoundaryValidated {
                    start: 12345,
                    end: Some(12345),
                    is_block_boundary: true,
                },
                12345,
                SlotType::Block,
            ),
            (
                Interval::Block(10),
                BoundaryValidated {
                    start: 12345,
                    end: Some(12355),
                    is_block_boundary: true,
                },
                12350,
                SlotType::Block,
            ),
            (
                Interval::Block(100),
                BoundaryValidated {
                    start: 12345,
                    end: Some(12355),
                    is_block_boundary: true,
                },
                12300,
                SlotType::Block,
            ),
            (
                Interval::Block(100),
                BoundaryValidated {
                    start: 12345,
                    end: Some(12300),
                    is_block_boundary: true,
                },
                0,
                SlotType::Block,
            ),
            (
                Interval::Block(100),
                BoundaryValidated {
                    start: 12345,
                    end: Some(12545),
                    is_block_boundary: true,
                },
                12400,
                SlotType::Block,
            ),
            (
                Interval::Block(100),
                BoundaryValidated {
                    start: 11345,
                    end: Some(11545),
                    is_block_boundary: true,
                },
                0,
                SlotType::Block,
            ),
            // wrong block interval
            (
                Interval::Block(100_000),
                BoundaryValidated {
                    start: 12345,
                    end: Some(12355),
                    is_block_boundary: true,
                },
                0,
                SlotType::Block,
            ),
            (
                Interval::Block(0),
                BoundaryValidated {
                    start: 12345,
                    end: Some(12355),
                    is_block_boundary: true,
                },
                0,
                SlotType::Block,
            ),
        ];

        // Check all these cases
        let env = mock_env();
        for (interval, boundary, outcome_block, outcome_slot_kind) in cases.iter() {
            let (next_id, slot_kind) = interval.next(&env, boundary, 1);
            assert_eq!(outcome_block, &next_id);
            assert_eq!(outcome_slot_kind, &slot_kind);
        }
    }

    #[test]
    fn interval_get_next_cron_time() {
        // (input, input, outcome, outcome)
        // test the case when slot_granularity_time == 1
        let cases: Vec<(Interval, BoundaryValidated, u64, SlotType)> = vec![
            (
                Interval::Cron("* * * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_797_419_879_305_533,
                    end: None,
                    is_block_boundary: false,
                },
                1_571_797_420_000_000_000, // current time in nanos is 1_571_797_419_879_305_533
                SlotType::Cron,
            ),
            (
                Interval::Cron("1 * * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_797_419_879_305_533,
                    end: None,
                    is_block_boundary: false,
                },
                1_571_797_441_000_000_000,
                SlotType::Cron,
            ),
            (
                Interval::Cron("* 0 * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_797_419_879_305_533,
                    end: None,
                    is_block_boundary: false,
                },
                1_571_799_600_000_000_000,
                SlotType::Cron,
            ),
            (
                Interval::Cron("15 0 * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_797_419_879_305_533,
                    end: None,
                    is_block_boundary: false,
                },
                1_571_799_615_000_000_000,
                SlotType::Cron,
            ),
            // with another start
            (
                Interval::Cron("15 0 * * * *".to_string()),
                BoundaryValidated {
                    start: 1_471_799_600_000_000_000,
                    end: None,
                    is_block_boundary: false,
                },
                1_571_799_615_000_000_000,
                SlotType::Cron,
            ),
            (
                Interval::Cron("15 0 * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_799_600_000_000_000,
                    end: None,
                    is_block_boundary: false,
                },
                1_571_799_615_000_000_000,
                SlotType::Cron,
            ),
            (
                Interval::Cron("15 0 * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_799_700_000_000_000,
                    end: None,
                    is_block_boundary: false,
                },
                1_571_803_215_000_000_000,
                SlotType::Cron,
            ),
            // cases when a boundary has end
            // current slot is the end slot
            (
                Interval::Cron("* * * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_797_419_879_305_533,
                    end: Some(1_571_797_419_879_305_533),
                    is_block_boundary: false,
                },
                1_571_797_419_879_305_533,
                SlotType::Cron,
            ),
            // the next slot is after the end, return end slot
            (
                Interval::Cron("* * * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_797_419_879_305_533,
                    end: Some(1_571_797_419_879_305_535),
                    is_block_boundary: false,
                },
                1_571_797_419_879_305_535,
                SlotType::Cron,
            ),
            // next slot in boundaries
            (
                Interval::Cron("* * * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_797_419_879_305_533,
                    end: Some(1_571_797_420_000_000_000),
                    is_block_boundary: false,
                },
                1_571_797_420_000_000_000,
                SlotType::Cron,
            ),
            // the task has ended
            (
                Interval::Cron("* * * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_797_419_879_305_533,
                    end: Some(1_571_797_419_879_305_532),
                    is_block_boundary: false,
                },
                0,
                SlotType::Cron,
            ),
            (
                Interval::Cron("15 0 * * * *".to_string()),
                BoundaryValidated {
                    start: 1_471_799_600_000_000_000,
                    end: Some(1_471_799_600_000_000_000),
                    is_block_boundary: false,
                },
                0,
                SlotType::Cron,
            ),
            (
                Interval::Cron("1 * * * * *".to_string()),
                BoundaryValidated {
                    start: 1_471_797_441_000_000_000,
                    end: Some(1_671_797_441_000_000_000),
                    is_block_boundary: false,
                },
                1_571_797_441_000_000_000,
                SlotType::Cron,
            ),
        ];
        // Check all these cases
        for (interval, boundary, outcome_time, outcome_slot_kind) in cases.iter() {
            let env = mock_env();
            let (next_id, slot_kind) = interval.next(&env, boundary, 1);
            assert_eq!(outcome_time, &next_id);
            assert_eq!(outcome_slot_kind, &slot_kind);
        }

        // slot_granularity_time == 120_000_000_000 ~ 2 minutes
        let cases: Vec<(Interval, BoundaryValidated, u64, SlotType)> = vec![
            (
                Interval::Cron("* * * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_797_419_879_305_533,
                    end: None,
                    is_block_boundary: false,
                },
                // the timestamp is in the current slot, so we take the next slot
                1_571_797_420_000_000_000_u64
                    .saturating_sub(1_571_797_420_000_000_000 % TWO_MINUTES)
                    + TWO_MINUTES, // current time in nanos is 1_571_797_419_879_305_533
                SlotType::Cron,
            ),
            (
                Interval::Cron("1 * * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_797_419_879_305_533,
                    end: None,
                    is_block_boundary: false,
                },
                1_571_797_440_000_000_000,
                SlotType::Cron,
            ),
            (
                Interval::Cron("* 0 * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_797_419_879_305_533,
                    end: None,
                    is_block_boundary: false,
                },
                1_571_799_600_000_000_000,
                SlotType::Cron,
            ),
            (
                Interval::Cron("15 0 * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_797_419_879_305_533,
                    end: None,
                    is_block_boundary: false,
                },
                1_571_799_600_000_000_000,
                SlotType::Cron,
            ),
            // with another start
            (
                Interval::Cron("15 0 * * * *".to_string()),
                BoundaryValidated {
                    start: 1_471_799_600_000_000_000,
                    end: None,
                    is_block_boundary: false,
                },
                1_571_799_600_000_000_000,
                SlotType::Cron,
            ),
            (
                Interval::Cron("15 0 * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_799_600_000_000_000,
                    end: None,
                    is_block_boundary: false,
                },
                1_571_799_600_000_000_000,
                SlotType::Cron,
            ),
            (
                Interval::Cron("15 0 * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_799_700_000_000_000,
                    end: None,
                    is_block_boundary: false,
                },
                1_571_803_200_000_000_000,
                SlotType::Cron,
            ),
            // cases when a boundary has end
            // boundary end in the current slot
            (
                Interval::Cron("* * * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_797_419_879_305_533,
                    end: Some(1_571_797_419_879_305_535),
                    is_block_boundary: false,
                },
                1_571_797_320_000_000_000,
                SlotType::Cron,
            ),
            // next slot in boundaries
            (
                Interval::Cron("1 * * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_797_419_879_305_533,
                    end: Some(1_571_797_560_000_000_000),
                    is_block_boundary: false,
                },
                1_571_797_440_000_000_000,
                SlotType::Cron,
            ),
            // next slot after the end, return end slot
            (
                Interval::Cron("1 * * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_797_419_879_305_533,
                    end: Some(1_571_797_420_000_000_000),
                    is_block_boundary: false,
                },
                1_571_797_320_000_000_000,
                SlotType::Cron,
            ),
            // the task has ended
            (
                Interval::Cron("* * * * * *".to_string()),
                BoundaryValidated {
                    start: 1_571_797_419_879_305_533,
                    end: Some(1_571_797_419_879_305_532),
                    is_block_boundary: false,
                },
                0,
                SlotType::Cron,
            ),
        ];
        // Check all these cases
        for (interval, boundary, outcome_time, outcome_slot_kind) in cases.iter() {
            let env = mock_env();
            let (next_id, slot_kind) = interval.next(&env, boundary, TWO_MINUTES);
            assert_eq!(outcome_time, &next_id);
            assert_eq!(outcome_slot_kind, &slot_kind);
        }
    }
}
