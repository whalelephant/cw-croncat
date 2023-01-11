#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use croncat_sdk_tasks::types::{BoundaryValidated, TaskRequest};
use cw2::set_contract_version;
use cw20::Cw20CoinVerified;

use crate::error::ContractError;
use crate::helpers::validate_boundary;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::MANAGER_ADDR;

const CONTRACT_NAME: &str = "croncat:croncat-tasks";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let validated = deps.api.addr_validate(&msg.manager_addr)?;
    MANAGER_ADDR.save(deps.storage, &validated)?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("manager_addr", validated))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // TODO: check config if paused
    match msg {
        ExecuteMsg::CreateTask { task } => execute_create_task(deps, env, info, task),
        ExecuteMsg::RemoveTask { task_hash } => todo!(),
        ExecuteMsg::RefillTaskBalance { task_hash } => todo!(),
        ExecuteMsg::RefillTaskCw20Balance {
            task_hash,
            cw20_coins,
        } => todo!(),
    }
}

fn execute_create_task(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    task: TaskRequest,
) -> Result<Response, ContractError> {
    let owner_id = &info.sender;
    // Validate cw20
    let verified_cw20 = task
        .cw20_coin
        .map(|cw20| -> StdResult<_> {
            Ok(Cw20CoinVerified {
                address: deps.api.addr_validate(&cw20.address)?,
                amount: cw20.amount,
            })
        })
        .transpose()?;

    // Validate boundary
    let boundary = validate_boundary(task.boundary)?;
    if !task.interval.is_valid() {
        return Err(ContractError::InvalidInterval {});
    }
    Ok(Response::new().add_attribute("action", "create_task"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    todo!();
}
