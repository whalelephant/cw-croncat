use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;

/// The response required by all queries. Bool is needed for croncat, T allows flexible rule engine
#[cw_serde]
pub struct QueryResponse<T = Binary> {
    pub result: bool,
    pub data: T,
}
