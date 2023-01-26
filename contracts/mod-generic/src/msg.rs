use crate::types::GenericQuery;
use cosmwasm_schema::{cw_serde, QueryResponses};
#[allow(unused_imports)]
use mod_sdk::types::QueryResponse;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Create a generic query
    // Parse the json-like result to get the required value using `gets`
    // Compare it to `value` according to `ordering`
    #[returns(QueryResponse)]
    GenericQuery(GenericQuery),
}
