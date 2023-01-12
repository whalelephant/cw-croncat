use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct OwnerOfNft {
    pub address: String,
    pub nft_address: String,
    pub token_id: String,
}
