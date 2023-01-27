use cosmwasm_std::{Deps, StdResult, Uint64};
use croncat_sdk_manager::msg::ManagerQueryMsg;
use croncat_sdk_manager::types::Config as ManagerConfig;

use croncat_sdk_tasks::msg::TasksQueryMsg;

pub fn query_manager_config(deps: Deps, manager_addr: String) -> StdResult<ManagerConfig> {
    // Get the denom from the manager contract
    let manager_config: ManagerConfig = deps
        .querier
        .query_wasm_smart(manager_addr, &ManagerQueryMsg::Config {})?;

    Ok(manager_config)
}

pub fn query_total_tasks(deps: Deps, tasks_addr: String) -> StdResult<u64> {
    // Get the denom from the manager contract
    let total_tasks: Uint64 = deps
        .querier
        .query_wasm_smart(tasks_addr, &TasksQueryMsg::TasksTotal {})?;

    Ok(total_tasks.u64())
}
