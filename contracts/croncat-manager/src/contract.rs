#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use croncat_sdk_core::types::UpdateConfig;
use cw2::set_contract_version;

use crate::balances::{
    add_available_native, execute_move_balances, execute_receive_cw20,
    execute_withdraw_wallet_balances, query_available_balances, query_cw20_wallet_balances,
};
use crate::error::ContractError;
use crate::helpers::check_ready_for_execution;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

pub(crate) const CONTRACT_NAME: &str = "crates.io:croncat-manager";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) const DEFAULT_NOMINATION_DURATION: u16 = 360;
/// Value based on non-wasm operations, wasm ops seem impossible to predict
pub(crate) const GAS_BASE_FEE: u64 = 300_000;
/// Gas needed for single action
pub(crate) const GAS_ACTION_FEE: u64 = 130_000;
/// Gas needed for single non-wasm query
pub(crate) const GAS_QUERY_FEE: u64 = 5_000;
/// Gas needed for single wasm query
pub(crate) const GAS_WASM_QUERY_FEE: u64 = 60_000;

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
        cw_rules_addr,
        croncat_tasks_addr,
        croncat_agents_addr,
        owner_id,
        gas_base_fee,
        gas_action_fee,
        gas_query_fee,
        gas_wasm_query_fee,
        gas_price,
        agent_nomination_duration,
    } = msg;

    let gas_price = gas_price.unwrap_or_default();
    // Make sure gas_price is valid
    if !gas_price.is_valid() {
        return Err(ContractError::InvalidGasPrice {});
    }

    let owner_id = owner_id
        .map(|human| deps.api.addr_validate(&human))
        .transpose()?
        .unwrap_or(info.sender);

    let config = Config {
        paused: false,
        owner_id,
        min_tasks_per_agent: 3,
        agents_eject_threshold: 600,
        agent_nomination_duration: agent_nomination_duration.unwrap_or(DEFAULT_NOMINATION_DURATION),
        cw_rules_addr: deps.api.addr_validate(&cw_rules_addr)?,
        croncat_tasks_addr: deps.api.addr_validate(&croncat_tasks_addr)?,
        croncat_agents_addr: deps.api.addr_validate(&croncat_agents_addr)?,
        agent_fee: 5,
        gas_price,
        gas_base_fee: gas_base_fee.map(Into::into).unwrap_or(GAS_BASE_FEE),
        gas_action_fee: gas_action_fee.map(Into::into).unwrap_or(GAS_ACTION_FEE),
        gas_query_fee: gas_query_fee.map(Into::into).unwrap_or(GAS_QUERY_FEE),
        gas_wasm_query_fee: gas_wasm_query_fee
            .map(Into::into)
            .unwrap_or(GAS_WASM_QUERY_FEE),
        slot_granularity_time: 10_000_000_000, // 10 seconds
        cw20_whitelist: vec![],
        native_denom: denom,
        balancer: Default::default(),
        limit: 100,
    };

    // Update state
    CONFIG.save(deps.storage, &config)?;
    for coin in info.funds {
        add_available_native(deps.storage, &coin)?;
    }
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        // TODO?:.add_attribute("config", format!("{:?}, &config"))
        .add_attribute("action", "instantiate")
        .add_attribute("paused", config.paused.to_string())
        .add_attribute("owner_id", config.owner_id.to_string())
        .add_attribute(
            "min_tasks_per_agent",
            config.min_tasks_per_agent.to_string(),
        )
        .add_attribute(
            "agents_eject_threshold",
            config.agents_eject_threshold.to_string(),
        )
        .add_attribute(
            "agent_nomination_duration",
            config.agent_nomination_duration.to_string(),
        )
        .add_attribute("cw_rules_addr", config.cw_rules_addr.to_string())
        .add_attribute("croncat_tasks_addr", config.croncat_tasks_addr.to_string())
        .add_attribute(
            "croncat_agents_addr",
            config.croncat_agents_addr.to_string(),
        )
        .add_attribute("agent_fee", config.agent_fee.to_string())
        .add_attribute("gas_price", format!("{:?}", config.gas_price))
        .add_attribute("gas_base_fee", config.gas_base_fee.to_string())
        .add_attribute("gas_action_fee", config.gas_action_fee.to_string())
        .add_attribute("gas_query_fee", config.gas_query_fee.to_string())
        .add_attribute("gas_wasm_query_fee", config.gas_wasm_query_fee.to_string())
        .add_attribute(
            "slot_granularity_time",
            config.slot_granularity_time.to_string(),
        )
        .add_attribute("cw20_whitelist", format!("{:?}", config.cw20_whitelist))
        .add_attribute("native_denom", config.native_denom.to_string())
        .add_attribute("balancer", format!("{:?}", config.balancer))
        .add_attribute("limit", config.limit.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig(msg) => execute_update_config(deps, info, msg),
        ExecuteMsg::MoveBalances {
            native_balances,
            cw20_balances,
            address,
        } => execute_move_balances(deps, info, native_balances, cw20_balances, address),
        ExecuteMsg::ProxyCall { task_hash: None } => execute_proxy_call(deps, env, info),
        ExecuteMsg::ProxyCall {
            task_hash: Some(task_hash),
        } => execute_proxy_call_with_queries(deps, env, info, task_hash),
        ExecuteMsg::Receive(msg) => execute_receive_cw20(deps, info, msg),
        ExecuteMsg::WithdrawCw20WalletBalances { cw20_amounts } => {
            execute_withdraw_wallet_balances(deps, info, cw20_amounts)
        }
        ExecuteMsg::Tick {} => execute_tick(deps, env, info),
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
            owner_id,
            slot_granularity_time,
            paused,
            agent_fee,
            gas_base_fee,
            gas_action_fee,
            gas_query_fee,
            gas_wasm_query_fee,
            gas_price,
            min_tasks_per_agent,
            agents_eject_threshold,
            balancer,
        } = msg;

        if info.sender != config.owner_id {
            return Err(ContractError::Unauthorized {});
        }

        let gas_price = gas_price.unwrap_or(config.gas_price);
        if !gas_price.is_valid() {
            return Err(ContractError::InvalidGasPrice {});
        }

        let owner_id = owner_id
            .map(|human| deps.api.addr_validate(&human))
            .transpose()?
            .unwrap_or(config.owner_id);

        let new_config = Config {
            paused: paused.unwrap_or(config.paused),
            owner_id,
            min_tasks_per_agent: min_tasks_per_agent.unwrap_or(config.min_tasks_per_agent),
            agents_eject_threshold: agents_eject_threshold.unwrap_or(config.agents_eject_threshold),
            agent_nomination_duration: config.agent_nomination_duration,
            cw_rules_addr: config.cw_rules_addr,
            croncat_tasks_addr: config.croncat_tasks_addr,
            croncat_agents_addr: config.croncat_agents_addr,
            agent_fee: agent_fee.unwrap_or(config.agent_fee),
            gas_price,
            gas_base_fee: gas_base_fee.unwrap_or(config.gas_base_fee),
            gas_action_fee: gas_action_fee.unwrap_or(config.gas_action_fee),
            gas_query_fee: gas_query_fee.unwrap_or(config.gas_query_fee),
            gas_wasm_query_fee: gas_wasm_query_fee.unwrap_or(config.gas_wasm_query_fee),
            slot_granularity_time: slot_granularity_time.unwrap_or(config.slot_granularity_time),
            cw20_whitelist: config.cw20_whitelist,
            native_denom: config.native_denom,
            balancer: balancer.unwrap_or(config.balancer),
            limit: config.limit,
        };
        Ok(new_config)
    })?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("config", format!("{new_config:?}")))
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
        QueryMsg::Cw20WalletBalances {
            wallet,
            from_index,
            limit,
        } => to_binary(&query_cw20_wallet_balances(
            deps, wallet, from_index, limit,
        )?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    todo!();
    //Ok(Response::new())
}
