use cosmwasm_schema::{cw_serde, QueryResponses};
use mod_sdk::types::QueryResponse;

use crate::types::OwnerOfNft;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(QueryResponse)]
    OwnerOfNft(OwnerOfNft),

    #[returns(QueryResponse)]
    AddrHasNft {
        address: String,
        nft_address: String,
    },
}
