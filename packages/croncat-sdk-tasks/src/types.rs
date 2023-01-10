use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint64, Timestamp, Empty, CosmosMsg};
use cw20::Cw20Coin;

#[cw_serde]
pub struct TaskRequest {
    pub interval: Interval,
    pub boundary: Option<Boundary>,
    pub stop_on_fail: bool,
    pub actions: Vec<Action>,
    // TODO: connect with queries modules
    // pub queries: Option<Vec<CroncatQuery>>,
    // pub transforms: Option<Vec<Transform>>,
    pub cw20_coins: Option<Vec<Cw20Coin>>,
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

pub struct BoundaryValidated {
    pub start: Option<u64>,
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
