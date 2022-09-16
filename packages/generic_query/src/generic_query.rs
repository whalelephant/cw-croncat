use crate::value_ordering::ValueOrdering;
use cosmwasm_std::Binary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct GenericQuery {
    pub contract_addr: String,
    pub msg: Binary,
    pub gets: Vec<ValueIndex>,

    pub ordering: ValueOrdering,
    pub value: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValueIndex {
    Key(String),
    Index(u64),
}
