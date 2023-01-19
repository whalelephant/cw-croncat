#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use croncat_sdk_core::types::UpdateConfig;
use cw2::set_contract_version;

use crate::balances::{
    add_available_native, add_user_native, execute_owner_withdraw, execute_receive_cw20,
    execute_user_withdraw, query_available_balances, query_users_balances,
};
use crate::error::ContractError;
use crate::helpers::check_ready_for_execution;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

pub(crate) const CONTRACT_NAME: &str = "crates.io:croncat-manager";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) const DEFAULT_NOMINATION_DURATION: u16 = 360;

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
        agent_nomination_duration,
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
        min_tasks_per_agent: 3,
        agents_eject_threshold: 600,
        agent_nomination_duration: agent_nomination_duration.unwrap_or(DEFAULT_NOMINATION_DURATION),
        croncat_factory_addr: deps.api.addr_validate(&croncat_factory_addr)?,
        croncat_tasks_key,
        croncat_agents_key,
        agent_fee: 5,
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
    for coin in info.funds {
        add_available_native(deps.storage, &coin)?;
    }
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
        ExecuteMsg::RefillNativeBalance {} => execute_refill_native_balance(deps, info),
        ExecuteMsg::UserWithdraw {
            native_balances,
            cw20_balances,
        } => execute_user_withdraw(deps, info, native_balances, cw20_balances),
        ExecuteMsg::Tick {} => execute_tick(deps, env, info),
    }
}

fn execute_refill_native_balance(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    for coin in info.funds {
        add_available_native(deps.storage, &coin)?;
        add_user_native(deps.storage, &info.sender, &coin)?;
    }
    Ok(Response::new().add_attribute("action", "refill_native_balance"))
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
            gas_price,
            min_tasks_per_agent,
            agents_eject_threshold,
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
            min_tasks_per_agent: min_tasks_per_agent.unwrap_or(config.min_tasks_per_agent),
            agents_eject_threshold: agents_eject_threshold.unwrap_or(config.agents_eject_threshold),
            agent_nomination_duration: config.agent_nomination_duration,
            croncat_factory_addr: config.croncat_factory_addr,
            croncat_tasks_key: croncat_tasks_key.unwrap_or(config.croncat_tasks_key),
            croncat_agents_key: croncat_agents_key.unwrap_or(config.croncat_agents_key),
            agent_fee: agent_fee.unwrap_or(config.agent_fee),
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::AvailableBalances { from_index, limit } => {
            to_binary(&query_available_balances(deps, from_index, limit)?)
        }
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
