use crate::types::HasBalanceComparator;
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
    /// Get native `address` balance with specific `denom`
    #[returns(mod_sdk::types::QueryResponse)]
    GetBalance { address: String, denom: String },
    /// Get cw20 balance by specific cw20 contract address
    #[returns(mod_sdk::types::QueryResponse)]
    GetCw20Balance {
        cw20_contract: String,
        address: String,
    },
    /// Compare balance of `address` (native or cw20) with `required_balance`
    #[returns(mod_sdk::types::QueryResponse)]
    HasBalanceComparator(HasBalanceComparator),
}
