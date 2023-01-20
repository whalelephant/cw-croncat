#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use croncat_sdk_tasks::types::{Config, TaskRequest};
use cw2::set_contract_version;
use cw20::Cw20CoinVerified;

use crate::error::ContractError;
use crate::helpers::{self, validate_boundary, validate_msg_calculate_usage};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::CONFIG;

const CONTRACT_NAME: &str = "croncat:croncat-tasks";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Default value based on non-wasm operations, wasm ops seem impossible to predict
// TODO: this values based of pre-split, need to recalculate GAS_BASE_FEE
pub(crate) const GAS_BASE_FEE: u64 = 300_000;
pub(crate) const GAS_ACTION_FEE: u64 = 130_000;
pub(crate) const GAS_QUERY_FEE: u64 = 130_000; // Load query module(~61_000) and query after that(~65_000+)
pub(crate) const GAS_LIMIT: u64 = 9_500_000; // 10M is default for juno, but let's make sure we have space for missed gas calculations
pub(crate) const SLOT_GRANULARITY_TIME: u64 = 10_000_000_000; // 10 seconds

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let InstantiateMsg {
        croncat_factory_addr,
        owner_addr,
        croncat_manager_key,
        croncat_agents_key,
        slot_granularity_time,
        gas_base_fee,
        gas_action_fee,
        gas_query_fee,
        gas_limit,
    } = msg;
    let config = Config {
        paused: false,
        owner_addr: owner_addr
            .map(|human| deps.api.addr_validate(&human))
            .transpose()?
            .unwrap_or(info.sender),
        croncat_factory_addr: deps.api.addr_validate(&croncat_factory_addr)?,
        croncat_manager_key,
        croncat_agents_key,
        slot_granularity_time: slot_granularity_time.unwrap_or(SLOT_GRANULARITY_TIME),
        gas_base_fee: gas_base_fee.unwrap_or(GAS_BASE_FEE),
        gas_action_fee: gas_action_fee.unwrap_or(GAS_ACTION_FEE),
        gas_query_fee: gas_query_fee.unwrap_or(GAS_QUERY_FEE),
        gas_limit: gas_limit.unwrap_or(GAS_LIMIT),
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
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
    }
}

fn execute_create_task(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    task: TaskRequest,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.paused {
        return Err(ContractError::Paused {});
    }
    let owner_id = &info.sender;

    // Validate boundary and interval
    let boundary = validate_boundary(env.block, &task.boundary, &task.interval)?;
    if !task.interval.is_valid() {
        return Err(ContractError::InvalidInterval {});
    }

    let amount_for_one_task = validate_msg_calculate_usage(
        deps.api,
        &task,
        &env.contract.address,
        &info.sender,
        &config,
    )?;

    // Validate cw20
    let verified_cw20 = task
        .cw20
        .map(|cw20| -> StdResult<_> {
            Ok(Cw20CoinVerified {
                address: deps.api.addr_validate(&cw20.address)?,
                amount: cw20.amount,
            })
        })
        .transpose()?;

    // TODO: pass message to manager with amount_for_one_task
    Ok(Response::new().add_attribute("action", "create_task"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    todo!();
}
