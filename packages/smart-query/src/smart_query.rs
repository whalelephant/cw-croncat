use cosmwasm_std::Binary;
use generic_query::{ValueIndex, ValueOrdering};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::SmartQueryError;
pub const PLACEHOLDER: &[u8] = b"$msg_ph";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SmartQueryHead {
    pub contract_addr: String,
    /// First query without placeholder!
    pub msg: Binary,
    /// Value from this message
    pub gets: Vec<ValueIndex>,

    pub queries: SmartQueries,

    pub ordering: ValueOrdering,
    pub value: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SmartQuery {
    pub contract_addr: String,
    pub msg: Binary,
    /// Value for the next iteration
    pub gets: Vec<ValueIndex>,
}

impl SmartQuery {
    /// Find and replace placeholder with a new value.
    // TODO: Discuss if we plan to do more than 1 replace
    pub fn replace_placeholder(&mut self, value: Binary) -> Result<(), SmartQueryError> {
        let mut msg = Vec::with_capacity(self.msg.0.len() + value.0.len());
        let pos = self
            .msg
            .0
            .windows(PLACEHOLDER.len())
            .position(|window| window == PLACEHOLDER)
            .ok_or(SmartQueryError::MissingPlaceholder {})?;
        msg.extend_from_slice(&self.msg.0[..pos]);
        msg.extend_from_slice(value.as_slice());
        msg.extend_from_slice(&self.msg.0[pos + PLACEHOLDER.len()..]);
        self.msg = Binary(msg);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SmartQueries(pub Vec<SmartQuery>);

impl SmartQueries {
    // TODO: Discuss if we need more than 1 replaces
    pub fn validate_queries(&self) -> bool {
        !self.0.is_empty()
            && self.0.iter().all(|q| {
                q.msg
                    .0
                    .windows(PLACEHOLDER.len())
                    .filter(|&window| window == PLACEHOLDER)
                    .count()
                    == 1
            })
    }
}
