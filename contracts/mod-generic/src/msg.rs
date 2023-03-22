use crate::types::{GenericQuery, CosmosQuery};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub version: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Create a generic query
    // Parse the json-like result to get the required value using `gets`
    // Compare it to `value` according to `ordering`
    #[returns(mod_sdk::types::QueryResponse)]
    GenericQuery(GenericQuery),

    // Batch queries for evaluating if task is ready or not
    // response data returned to caller
    #[returns(mod_sdk::types::QueryResponse)]
    BatchQuery {
        queries: Vec<CosmosQuery>,
    },
}
