use crate::types::HasBalanceComparator;
use cosmwasm_schema::{cw_serde, QueryResponses};
use mod_sdk::types::QueryResponse;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Individual query evaluations
    #[returns(QueryResponse)]
    GetBalance { address: String, denom: String },
    #[returns(QueryResponse)]
    GetCw20Balance {
        cw20_contract: String,
        address: String,
    },
    #[returns(QueryResponse)]
    HasBalanceComparator(HasBalanceComparator),
}
