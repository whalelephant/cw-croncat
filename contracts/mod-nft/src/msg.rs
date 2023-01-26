use cosmwasm_schema::{cw_serde, QueryResponses};
#[allow(unused_imports)]
use mod_sdk::types::QueryResponse;

use crate::types::OwnerOfNft;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Check if `address` is the owner of the token
    #[returns(QueryResponse)]
    OwnerOfNft(OwnerOfNft),

    /// Check if `address` owns any tokens on `nft_address` contract
    #[returns(QueryResponse)]
    AddrHasNft {
        address: String,
        nft_address: String,
    },
}
