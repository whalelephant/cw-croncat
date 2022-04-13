use cosmwasm_std::{Env, Timestamp};
use cron_schedule::Schedule;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum BoundarySpec {
    /// Represents the block height
    Height(u64),

    /// Represents the block timestamp
    Time(Timestamp),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Boundary {
    ///
    pub start: Option<BoundarySpec>,
    ///
    pub end: Option<BoundarySpec>,
}

fn get_next_block_limited(env: Env, boundary: Boundary) -> (u64, SlotType) {
    // immediate needs to return this block + 1
    let current_block_height = env.block.height;

    let next_block_height = if boundary.start.is_some() {
        match boundary.start.unwrap() {
            // Note: Not bothering with time, as that should get handled with the cron situations,
            // and probably throw an error when mixing blocks and cron
            BoundarySpec::Height(id) => {
                if current_block_height < id {
                    id
                } else {
                    current_block_height
                }
            }
            _ => current_block_height,
        }
    } else {
        current_block_height
    };

    (next_block_height + 1, SlotType::Block)
}

// So either:
// - Boundary specifies a start/end that block offsets can be computed from
// - Block offset will truncate to specific modulo offsets
fn get_next_block_by_offset(env: Env, boundary: Boundary, block: u64) -> (u64, SlotType) {
    let current_block_height = env.block.height;
    let modulo_block = current_block_height.saturating_sub(current_block_height % block) + block;

    let next_block_height = if boundary.start.is_some() {
        match boundary.start.unwrap() {
            // Note: Not bothering with time, as that should get handled with the cron situations,
            // and probably throw an error when mixing blocks and cron
            BoundarySpec::Height(id) => {
                if current_block_height < id {
                    id.saturating_sub(id % block)
                } else {
                    modulo_block
                }
            }
            _ => modulo_block,
        }
    } else {
        modulo_block
    };

    (next_block_height, SlotType::Block)
}

// TODO: Change this to ext pkg
impl Interval {
    pub fn next(&self, env: Env, boundary: Boundary) -> (u64, SlotType) {
        match self {
            // return the first block within a specific range that can be triggered 1 time.
            Interval::Once => get_next_block_limited(env, boundary),
            // return the first block within a specific range that can be triggered immediately, potentially multiple times.
            Interval::Immediate => get_next_block_limited(env, boundary),
            // return the first block within a specific range that can be triggered 1 or more times based on timestamps.
            // Uses crontab spec
            Interval::Cron(crontab) => {
                let current_block_ts: u64 = env.block.time.nanos();
                // TODO: get current timestamp within boundary
                let current_ts: u64 = if boundary.start.is_some() {
                    let start = boundary.start.unwrap();

                    match start {
                        // Note: Not bothering with height, as that should get handled with the block situations,
                        // and probably throw an error when mixing blocks and cron
                        BoundarySpec::Time(ts) => {
                            if current_block_ts < ts.nanos() {
                                ts.nanos()
                            } else {
                                current_block_ts
                            }
                        }
                        _ => current_block_ts,
                    }
                } else {
                    current_block_ts
                };

                let schedule = Schedule::from_str(crontab.as_str()).unwrap();
                let next_ts = schedule.next_after(&current_ts).unwrap();
                (next_ts, SlotType::Cron)
            }
            // return the block within a specific range that can be triggered 1 or more times based on block heights.
            // Uses block offset (Example: Block(100) will trigger every 100 blocks)
            // So either:
            // - Boundary specifies a start/end that block offsets can be computed from
            // - Block offset will truncate to specific modulo offsets
            Interval::Block(block) => get_next_block_by_offset(env, boundary, *block),
        }
    }
    pub fn slot_id_from(&self, time: u64) -> u64 {
        // TODO: need config param for the slot size
        // round the timestamp down to slot granularity
        time
    }
}

#[derive(Debug, PartialEq)]
pub enum SlotType {
    Block,
    Cron,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Slot {
    tasks: Vec<Vec<u8>>,
}

// TODO: Add/remove
impl Slot {
    pub fn new(&self, tasks: Option<Vec<Vec<u8>>>) -> Self {
        Self {
            tasks: tasks.unwrap_or_default(),
        }
    }
    // pub fn from_timestamp(ts: Timestamp, slot_granularity: u64) -> u64 {
    //     // Round down to slot granularity
    //     // let slot_remainder = block % slot_granularity;
    //     // block.saturating_sub(slot_remainder)
    // let schedule = Schedule::from_str(&crontab).unwrap();
    // let next_ts = schedule.next_after(&current_ts).unwrap();
    // }
    // pub fn push(&mut self, task_hash: String) {
    //     self.tasks.push(task_hash);
    // }
    // pub fn remove(&mut self, task_hash: String) {
    //     self.tasks.push(task_hash);
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_env;

    #[test]
    fn interval_get_next_block_by_offset() {
        let env = mock_env();
        let interval = Interval::Block(10);
        let boundary = Boundary {
            start: None,
            end: None,
        };

        // CHECK IT!
        let (next_id, slot_kind) = interval.next(env, boundary);
        println!("next_id {:?}, slot_kind {:?}", next_id, slot_kind);
        assert_eq!(12350, next_id);
        assert_eq!(SlotType::Block, slot_kind);
    }
}
