use cosmwasm_std::{QuerierWrapper, QueryRequest, StdResult, WasmQuery};
use cw2::ContractVersion;

pub struct CroncatQuerier<'a> {
    querier: &'a QuerierWrapper<'a>,
}
impl<'a> CroncatQuerier<'a> {
    pub fn new(querier: &'a QuerierWrapper<'a>) -> Self {
        CroncatQuerier { querier }
    }

    pub fn query_contract_info(&self, contract_address: String) -> StdResult<ContractVersion> {
        let req = QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: contract_address,
            key: cosmwasm_std::Binary::from(b"contract_info"),
        });
        self.querier.query(&req)
    }
}
