use cosmwasm_std::{Addr, Deps, Empty, QuerierWrapper, StdError, StdResult, Uint64};
use croncat_sdk_agents::types::Config;
use croncat_sdk_manager::msg::ManagerQueryMsg;
use croncat_sdk_manager::types::Config as ManagerConfig;

use crate::error::ContractError;
use croncat_factory::state::CONTRACT_ADDRS;
use croncat_sdk_tasks::msg::TasksQueryMsg;

pub fn query_manager_config(deps: Deps, config: &Config) -> StdResult<ManagerConfig> {
    let manager_addr = query_manager_addr(&deps.querier, config)?;
    // Get the denom from the manager contract
    let manager_config: ManagerConfig = deps
        .querier
        .query_wasm_smart(manager_addr, &ManagerQueryMsg::Config {})?;

    Ok(manager_config)
}

pub fn query_total_tasks(deps: Deps, config: &Config) -> StdResult<u64> {
    let tasks_addr = query_tasks_addr(&deps.querier, config)?;
    // Get the denom from the manager contract
    let total_tasks: Uint64 = deps
        .querier
        .query_wasm_smart(tasks_addr, &TasksQueryMsg::TasksTotal {})?;

    Ok(total_tasks.u64())
}

pub(crate) fn assert_caller_is_tasks_contract(
    deps_queries: &QuerierWrapper<Empty>,
    config: &Config,
    sender: &Addr,
) -> StdResult<()> {
    let addr = query_tasks_addr(deps_queries, config)?;
    if addr != *sender {
        return Err(cosmwasm_std::StdError::GenericErr {
            msg: ContractError::Unauthorized {}.to_string(),
        });
    }
    Ok(())
}

pub(crate) fn query_manager_addr(
    deps_queries: &QuerierWrapper<Empty>,
    config: &Config,
) -> StdResult<Addr> {
    let (manager_name, version) = &config.croncat_manager_key;
    CONTRACT_ADDRS
        .query(
            deps_queries,
            config.croncat_factory_addr.clone(),
            (manager_name, version),
        )?
        .ok_or(cosmwasm_std::StdError::GenericErr {
            msg: ContractError::InvalidVersionKey {}.to_string(),
        })
}

pub(crate) fn query_tasks_addr(
    deps_queries: &QuerierWrapper<Empty>,
    config: &Config,
) -> StdResult<Addr> {
    let (agents_name, version) = &config.croncat_tasks_key;
    CONTRACT_ADDRS
        .query(
            deps_queries,
            config.croncat_factory_addr.clone(),
            (agents_name, version),
        )?
        .ok_or_else(|| StdError::generic_err(
            ContractError::InvalidVersionKey {}.to_string(),
        ))
}
