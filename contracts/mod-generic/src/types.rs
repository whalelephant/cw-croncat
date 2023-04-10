use crate::value_ordering::ValueOrdering;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, StdError, StdResult, WasmQuery};
use serde_cw_value::Value;

#[cw_serde]
pub struct GenericQuery {
    pub contract_addr: String,
    pub msg: Binary,
    pub path_to_value: PathToValue,

    pub ordering: ValueOrdering,
    pub value: Binary,
}

/// Query given module contract with a message
#[cw_serde]
pub struct CroncatQuery {
    /// This is address of the queried module contract.
    /// For the addr can use one of our croncat-mod-* contracts, or custom contracts
    pub contract_addr: String,
    pub msg: Binary,
    /// For queries with `check_result`: query return value should be formatted as a:
    /// [`QueryResponse`](mod_sdk::types::QueryResponse)
    pub check_result: bool,
}

/// Query given module contract with a message
#[cw_serde]
pub enum CosmosQuery<T = WasmQuery> {
    // For optionally checking results, esp for modules
    Croncat(CroncatQuery),

    // For covering native wasm query cases (Smart, Raw)
    Wasm(T),
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
        // If empty pointer, return the entirety
        if self.0.is_empty() {
            return Ok(val);
        }

        // Follow the path of the pointer
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
