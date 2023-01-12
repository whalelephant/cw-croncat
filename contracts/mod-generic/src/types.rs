use crate::value_ordering::ValueOrdering;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, StdError, StdResult};
use serde_cw_value::Value;

#[cw_serde]
pub struct GenericQuery {
    pub contract_addr: String,
    pub msg: Binary,
    pub path_to_value: PathToValue,

    pub ordering: ValueOrdering,
    pub value: Binary,
}

// TODO Implement Serialize Deserialize https://github.com/CosmWasm/serde-json-wasm/issues/43
#[cw_serde]
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

#[cw_serde]
pub struct PathToValue(pub Vec<ValueIndex>);

impl From<Vec<ValueIndex>> for PathToValue {
    fn from(path: Vec<ValueIndex>) -> Self {
        PathToValue(path)
    }
}

impl PathToValue {
    /// Find the value by the "key" path
    pub fn find_value<'a>(&self, val: &'a mut Value) -> StdResult<&'a mut Value> {
        let mut current_val = val;
        for get in self.0.iter() {
            match get {
                ValueIndex::Key(s) => {
                    if let Value::Map(map) = current_val {
                        current_val = map
                            .get_mut(&Value::String(s.clone()))
                            .ok_or_else(|| StdError::generic_err("Invalid key for value"))?;
                    } else {
                        return Err(StdError::generic_err("Failed to get map from this value"));
                    }
                }
                ValueIndex::Index(n) => {
                    if let Value::Seq(seq) = current_val {
                        current_val = seq
                            .get_mut(*n as usize)
                            .ok_or_else(|| StdError::generic_err("Invalid index for value"))?;
                    } else {
                        return Err(StdError::generic_err(
                            "Failed to get sequence from this value",
                        ));
                    }
                }
            }
        }
        Ok(current_val)
    }
}
