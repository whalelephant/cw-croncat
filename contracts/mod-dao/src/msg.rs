use cosmwasm_schema::{cw_serde, QueryResponses};
use mod_sdk::types::QueryResponse;

use crate::types::ProposalStatusMatches;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Query proposal status and compare it to pre-defined status
    #[returns(QueryResponse)]
    ProposalStatusMatches(ProposalStatusMatches),

    // Query proposals and check if there're any passed proposals
    #[returns(QueryResponse)]
    HasPassedProposals { dao_address: String },

    // Query proposals and check if there're any passed proposals with Wasm::Migration message
    #[returns(QueryResponse)]
    HasPassedProposalWithMigration { dao_address: String },

    // Check if the last proposal id is greater than specified value
    #[returns(QueryResponse)]
    HasProposalsGtId { dao_address: String, value: u64 },
}
