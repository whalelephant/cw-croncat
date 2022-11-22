use cosmwasm_std::Binary;
use generic_query::{PathToValue, ValueOrdering};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SmartQueryHead {
    pub contract_addr: String,
    /// First query without placeholder!
    pub msg: Binary,
    /// Value from this message
    pub path_to_query_value: PathToValue,

    pub queries: SmartQueries,

    pub ordering: ValueOrdering,
    pub value: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SmartQuery {
    pub contract_addr: String,
    pub msg: Binary,
    /// Replace value inside this query
    pub path_to_msg_value: PathToValue,
    /// Value passed to the next iteration
    pub path_to_query_value: PathToValue,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SmartQueries(pub Vec<SmartQuery>);
