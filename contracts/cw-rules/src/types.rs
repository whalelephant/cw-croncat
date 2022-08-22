use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::helpers::ValueOrdering;
use cosmwasm_std::Binary;
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct GenericQuery {
    pub msg: Binary,
    /// we support up to one value in depth
    pub gets: Vec<ValueIndex>,

    pub ordering: ValueOrdering,
    pub value: Value,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(untagged)]
pub enum ValueIndex {
    Key(String),
    Number(u64),
}
