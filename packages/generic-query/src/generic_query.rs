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

// TODO Implement Serialize Deserialize https://github.com/CosmWasm/serde-json-wasm/issues/43
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValueIndex {
    Key(String),
    Index(u64),
}

impl From<u64> for ValueIndex {
    fn from(i: u64) -> Self {
        Self::Index(i)
    }
}

impl From<String> for ValueIndex {
    fn from(s: String) -> Self {
        Self::Key(s)
    }
}
