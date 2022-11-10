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
        let new_key: cosmwasm_std::Binary =
            [99, 111, 110, 116, 114, 97, 99, 116, 95, 105, 110, 102, 111].into();
        let req = QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: contract_address,
            key: new_key,
        });
        self.querier.query(&req)
    }
}
