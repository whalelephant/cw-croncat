#![allow(unused)]

use cosmwasm_std::Empty;
use cw_multi_test::{Contract, ContractWrapper};

pub(crate) fn croncat_manager_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply);
    Box::new(contract)
}

pub(crate) fn croncat_factory_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        croncat_factory::contract::execute,
        croncat_factory::contract::instantiate,
        croncat_factory::contract::query,
    )
    .with_reply(croncat_factory::contract::reply);
    Box::new(contract)
}

pub(crate) fn croncat_tasks_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        croncat_tasks::contract::execute,
        croncat_tasks::contract::instantiate,
        croncat_tasks::contract::query,
    );
    Box::new(contract)
}

pub(crate) fn croncat_agents_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        croncat_agents::contract::execute,
        croncat_agents::contract::instantiate,
        croncat_agents::contract::query,
    );
    Box::new(contract)
}

pub(crate) fn cw20_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

pub(crate) fn mod_balances_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        croncat_mod_balances::contract::execute,
        croncat_mod_balances::contract::instantiate,
        croncat_mod_balances::contract::query,
    );
    Box::new(contract)
}

pub(crate) fn mod_generic_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        croncat_mod_generic::contract::execute,
        croncat_mod_generic::contract::instantiate,
        croncat_mod_generic::contract::query,
    );
    Box::new(contract)
}

pub(crate) fn cw_boolean_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw_boolean_contract::entry_points::execute::execute,
        cw_boolean_contract::entry_points::instantiate::instantiate,
        cw_boolean_contract::entry_points::query::query,
    );
    Box::new(contract)
}
