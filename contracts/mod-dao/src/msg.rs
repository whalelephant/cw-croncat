use cosmwasm_schema::{cw_serde, QueryResponses};
use mod_sdk::types::QueryResponse;

use crate::types::CheckProposalStatus;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(QueryResponse)]
    CheckProposalStatus(CheckProposalStatus),

    #[returns(QueryResponse)]
    CheckPassedProposals { dao_address: String },
}
