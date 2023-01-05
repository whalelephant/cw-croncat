use cosmwasm_std::Empty;
use cw_multi_test::{ContractWrapper, Contract};


pub(crate) fn croncat_manager_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    ).with_reply(crate::contract::reply);
    Box::new(contract)
}