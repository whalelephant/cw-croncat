#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, Uint128,
};
use croncat_sdk_core::internal_messages::manager::ManagerCreateTaskBalance;
use croncat_sdk_manager::types::{TaskBalance, UpdateConfig};
use cw2::set_contract_version;

use crate::balances::{
    execute_owner_withdraw, execute_receive_cw20, execute_refill_native_balance,
    execute_refill_task_cw20, execute_user_withdraw, query_users_balances, sub_user_cw20,
};
use crate::error::ContractError;
use crate::helpers::{
    attached_natives, calculate_required_natives, check_if_sender_is_tasks,
    check_ready_for_execution, gas_with_fees,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, TASKS_BALANCES, TREASURY_BALANCE};

pub(crate) const CONTRACT_NAME: &str = "crates.io:croncat-manager";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) const DEFAULT_FEE: u64 = 5;

/// Instantiate
/// First contract method before it runs on the chains
/// See [`InstantiateMsg`] for more details
/// `gas_price` and `owner_id` getting validated
///
/// Response: every [`Config`] field as attributes
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Deconstruct so we don't miss fields
    let InstantiateMsg {
        denom,
        croncat_factory_addr,
        croncat_tasks_key,
        croncat_agents_key,
        owner_addr,
        gas_price,
        treasury_addr,
    } = msg;

    let gas_price = gas_price.unwrap_or_default();
    // Make sure gas_price is valid
    if !gas_price.is_valid() {
        return Err(ContractError::InvalidGasPrice {});
    }

    let owner_addr = owner_addr
        .map(|human| deps.api.addr_validate(&human))
        .transpose()?
        .unwrap_or(info.sender);

    let config = Config {
        paused: false,
        owner_addr,
        croncat_factory_addr: deps.api.addr_validate(&croncat_factory_addr)?,
        croncat_tasks_key,
        croncat_agents_key,
        agent_fee: DEFAULT_FEE,
        treasury_fee: DEFAULT_FEE,
        gas_price,
        cw20_whitelist: vec![],
        native_denom: denom,
        limit: 100,
        treasury_addr: treasury_addr
            .map(|human| deps.api.addr_validate(&human))
            .transpose()?,
    };

    // Update state
    CONFIG.save(deps.storage, &config)?;
    TREASURY_BALANCE.save(deps.storage, &Uint128::zero())?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("paused", config.paused.to_string())
        .add_attribute("owner_id", config.owner_addr.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig(msg) => execute_update_config(deps, info, *msg),
        ExecuteMsg::OwnerWithdraw {} => execute_owner_withdraw(deps, info),
        ExecuteMsg::ProxyCall { task_hash: None } => execute_proxy_call(deps, env, info),
        ExecuteMsg::ProxyCall {
            task_hash: Some(task_hash),
        } => execute_proxy_call_with_queries(deps, env, info, task_hash),
        ExecuteMsg::Receive(msg) => execute_receive_cw20(deps, info, msg),
        ExecuteMsg::RefillTaskBalance { task_hash } => {
            execute_refill_native_balance(deps, info, task_hash)
        }
        ExecuteMsg::RefillTaskCw20Balance { task_hash, cw20 } => {
            execute_refill_task_cw20(deps, info, task_hash, cw20)
        }
        ExecuteMsg::UserWithdraw { limit } => execute_user_withdraw(deps, info, limit),
        ExecuteMsg::Tick {} => execute_tick(deps, env, info),
        // TODO: make method ONLY for tasks contract to create task_hash's balance!
        ExecuteMsg::CreateTaskBalance(msg) => execute_create_task_balance(deps, info, msg),
    }
}

fn execute_proxy_call(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    check_ready_for_execution(&info, &config)?;

    // TODO: query agent to check if ready
    // TODO: execute task

    Ok(Response::new().add_attribute("action", "proxy_call"))
}

fn execute_proxy_call_with_queries(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _task_hash: String,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    check_ready_for_execution(&info, &config)?;

    // TODO: query agent to check if ready
    // TODO: execute task

    Ok(Response::new().add_attribute("action", "proxy_call_with_queries"))
}

/// Execute: UpdateConfig
/// Used by contract owner to update config or pause contract
///
/// Returns updated [`Config`]
pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    msg: UpdateConfig,
) -> Result<Response, ContractError> {
    let new_config = CONFIG.update(deps.storage, |config| {
        // Deconstruct, so we don't miss any fields
        let UpdateConfig {
            owner_addr,
            paused,
            agent_fee,
            treasury_fee,
            gas_price,
            croncat_tasks_key,
            croncat_agents_key,
            treasury_addr,
        } = msg;

        if info.sender != config.owner_addr {
            return Err(ContractError::Unauthorized {});
        }

        let gas_price = gas_price.unwrap_or(config.gas_price);
        if !gas_price.is_valid() {
            return Err(ContractError::InvalidGasPrice {});
        }

        let owner_addr = owner_addr
            .map(|human| deps.api.addr_validate(&human))
            .transpose()?
            .unwrap_or(config.owner_addr);
        let treasury_addr = if let Some(human) = treasury_addr {
            Some(deps.api.addr_validate(&human)?)
        } else {
            config.treasury_addr
        };

        let new_config = Config {
            paused: paused.unwrap_or(config.paused),
            owner_addr,
            croncat_factory_addr: config.croncat_factory_addr,
            croncat_tasks_key: croncat_tasks_key.unwrap_or(config.croncat_tasks_key),
            croncat_agents_key: croncat_agents_key.unwrap_or(config.croncat_agents_key),
            agent_fee: agent_fee.unwrap_or(config.agent_fee),
            treasury_fee: treasury_fee.unwrap_or(config.treasury_fee),
            gas_price,
            cw20_whitelist: config.cw20_whitelist,
            native_denom: config.native_denom,
            limit: config.limit,
            treasury_addr,
        };
        Ok(new_config)
    })?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("action", "instantiate")
        .add_attribute("paused", new_config.paused.to_string())
        .add_attribute("owner_id", new_config.owner_addr.to_string()))
}

/// Execute: UpdateConfig
/// Helps manage and cleanup agents
/// Deletes agents which missed more than agents_eject_threshold slot
///
/// Returns removed agents
// TODO: It might be not possible to deserialize all of the active agents, need to find better solution
// See issue #247
pub fn execute_tick(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    check_ready_for_execution(&info, &config)?;

    // let current_slot = env.block.height;
    // let cfg = CONFIG.load(deps.storage)?;
    // let mut attributes = vec![];
    // let mut submessages = vec![];

    // for agent_id in self.agent_active_queue.load(deps.storage)? {
    //     let agent = self.agents.load(deps.storage, &agent_id)?;
    //     if current_slot > agent.last_executed_slot + cfg.agents_eject_threshold {
    //         let resp = self
    //             .unregister_agent(deps.storage, &agent_id, None)
    //             .unwrap_or_default();
    //         // Save attributes and messages
    //         attributes.extend_from_slice(&resp.attributes);
    //         submessages.extend_from_slice(&resp.messages);
    //     }
    // }

    // // Check if there isn't any active or pending agents
    // if self.agent_active_queue.load(deps.storage)?.is_empty()
    //     && self.agent_pending_queue.is_empty(deps.storage)?
    // {
    //     attributes.push(Attribute::new("lifecycle", "tick_failure"))
    // }
    Ok(Response::new().add_attribute("action", "tick"))
}

fn execute_create_task_balance(
    deps: DepsMut,
    info: MessageInfo,
    msg: ManagerCreateTaskBalance,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    check_if_sender_is_tasks(&deps.querier, &config, &info.sender)?;
    let (native, ibc) = attached_natives(&config.native_denom, info.funds)?;
    let cw20 = msg.cw20;
    if let Some(attached_cw20) = &cw20 {
        sub_user_cw20(deps.storage, &msg.sender, attached_cw20)?;
    }
    let tasks_balance = TaskBalance {
        native_balance: native,
        cw20_balance: cw20,
        ibc_balance: ibc,
    };
    // Let's check if task has enough attached balance
    {
        let gas_with_fees = gas_with_fees(
            msg.amount_for_one_task.gas,
            config.agent_fee + config.treasury_fee,
        )?;
        let native_for_gas_required = config.gas_price.calculate(gas_with_fees)?;

        let (native_for_sends_required, ibc_required) =
            calculate_required_natives(msg.amount_for_one_task.coin, &config.native_denom)?;
        tasks_balance.verify_enough_attached(
            Uint128::from(native_for_gas_required) + native_for_sends_required,
            msg.amount_for_one_task.cw20,
            ibc_required,
            msg.recurring,
            &config.native_denom,
        )?;
    }
    TASKS_BALANCES.save(deps.storage, &msg.task_hash, &tasks_balance)?;

    Ok(Response::new().add_attribute("action", "create_task_balance"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::TreasuryBalance {} => to_binary(&TREASURY_BALANCE.load(deps.storage)?),
        QueryMsg::UsersBalances {
            wallet,
            from_index,
            limit,
        } => to_binary(&query_users_balances(deps, wallet, from_index, limit)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    todo!();
    //Ok(Response::new())
}
