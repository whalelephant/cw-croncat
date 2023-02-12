#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::external::*;
use crate::msg::*;
use crate::state::*;
use cosmwasm_std::{
    has_coins, to_binary, Addr, Attribute, Binary, Coin, Deps, DepsMut, Empty, Env, MessageInfo,
    QuerierWrapper, Response, StdError, StdResult, Storage, Uint64,
};
use croncat_sdk_agents::msg::{
    AgentResponse, AgentTasksResponse, GetAgentIdsResponse, UpdateConfig,
};
use croncat_sdk_agents::types::{AgentInfo, Config};
use croncat_sdk_core::internal_messages::agents::{AgentOnTaskCompleted, AgentOnTaskCreated};
use cw2::set_contract_version;

pub(crate) const CONTRACT_NAME: &str = "crate:croncat-agents";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let InstantiateMsg {
        owner_addr,
        version,
        croncat_manager_key,
        croncat_tasks_key,
        agent_nomination_duration,
        min_tasks_per_agent,
        min_coin_for_agent_registration,
        max_slot_passover,
        min_active_reserve,
    } = msg;

    let owner_addr = owner_addr
        .map(|human| deps.api.addr_validate(&human))
        .transpose()?
        .unwrap_or_else(|| info.sender.clone());

    let config = &Config {
        min_tasks_per_agent: min_tasks_per_agent.unwrap_or(DEFAULT_MIN_TASKS_PER_AGENT),
        croncat_factory_addr: info.sender,
        croncat_manager_key,
        croncat_tasks_key,
        agent_nomination_block_duration: agent_nomination_duration
            .unwrap_or(DEFAULT_NOMINATION_BLOCK_DURATION),
        owner_addr,
        paused: false,
        max_slot_passover: max_slot_passover.unwrap_or(DEFAULT_MAX_SLOTS_PASSOVER),
        min_coins_for_agent_registration: min_coin_for_agent_registration
            .unwrap_or(DEFAULT_MIN_COINS_FOR_AGENT_REGISTRATION),
        min_active_reserve: min_active_reserve.unwrap_or(DEFAULT_MIN_ACTIVE_RESERVE),
    };

    AGENT_DISTRIBUTOR.reset_nomination_checkpoint(deps.storage)?;
    CONFIG.save(deps.storage, config)?;

    set_contract_version(
        deps.storage,
        CONTRACT_NAME,
        version.unwrap_or_else(|| CONTRACT_VERSION.to_string()),
    )?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("paused", config.paused.to_string())
        .add_attribute("owner", config.owner_addr.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAgent { account_id } => to_binary(&query_get_agent(deps, account_id)?),
        QueryMsg::GetAgentIds { from_index, limit } => {
            to_binary(&query_get_agent_ids(deps, from_index, limit)?)
        }
        QueryMsg::GetAgentTasks { account_id } => to_binary(&query_agent_tasks(deps, account_id)?),
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterAgent { payable_account_id } => {
            register_agent(deps, info, env, payable_account_id)
        }
        ExecuteMsg::UnregisterAgent {} => {
            unregister_agent(deps.storage, &deps.querier, &info.sender)
        }
        ExecuteMsg::UpdateAgent { payable_account_id } => {
            update_agent(deps, info, payable_account_id)
        }
        ExecuteMsg::CheckInAgent {} => accept_nomination_agent(deps, info, env),
        ExecuteMsg::OnTaskCreated(msg) => on_task_created(env, deps, info, msg),
        ExecuteMsg::UpdateConfig { config } => execute_update_config(deps, info, config),
        ExecuteMsg::Tick {} => execute_tick(deps, env),
        ExecuteMsg::OnTaskCompleted(msg) => on_task_completed(deps, &env, info, msg),
    }
}

fn query_get_agent(deps: Deps, account_id: String) -> StdResult<AgentResponse> {
    let account_id = deps.api.addr_validate(&account_id)?;

    let agent_result = AGENT_DISTRIBUTOR
        .get_agent(deps.storage, &account_id)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    let agent = if let Some(a) = agent_result {
        a
    } else {
        return Ok(AgentResponse { agent: None });
    };

    let config: Config = CONFIG.load(deps.storage)?;
    let rewards =
        croncat_manager_contract::query_agent_rewards(&deps.querier, &config, account_id.as_str())?;

    let agent_response = AgentResponse {
        agent: Some(AgentInfo {
            status: agent.status,
            payable_account_id: agent.payable_account_id,
            balance: rewards,
            last_executed_slot: agent.last_executed_slot,
            register_start: agent.register_start,
        }),
    };
    Ok(agent_response)
}

/// Get a list of agent addresses
fn query_get_agent_ids(
    deps: Deps,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> StdResult<GetAgentIdsResponse> {
    let (active, pending) = AGENT_DISTRIBUTOR
        .get_agent_ids(deps.storage, from_index, limit)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    Ok(GetAgentIdsResponse { active, pending })
}

fn query_agent_tasks(deps: Deps, agent_id: String) -> StdResult<AgentTasksResponse> {
    let account_id = deps.api.addr_validate(&agent_id)?;
    let cfg: Config = CONFIG.load(deps.storage)?;

    let (block_slots, cron_slots) = croncat_tasks_contract::query_tasks_slots(deps, &cfg)?;
    if block_slots == 0 && cron_slots == 0 {
        return Ok(AgentTasksResponse {
            total_block_tasks: Uint64::zero(),
            total_cron_tasks: Uint64::zero(),
        });
    }
    let result = AGENT_DISTRIBUTOR
        .get_available_tasks(deps.storage, &account_id, (block_slots, cron_slots))
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    Ok(AgentTasksResponse {
        total_block_tasks: Uint64::from(result.0),
        total_cron_tasks: Uint64::from(result.1),
    })
}

/// Add any account as an agent that will be able to execute tasks.
/// Registering allows for rewards accruing with micro-payments which will accumulate to more long-term.
///
/// Optional Parameters:
/// "payable_account_id" - Allows a different account id to be specified, so a user can receive funds at a different account than the agent account.
fn register_agent(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    payable_account_id: Option<String>,
) -> Result<Response, ContractError> {
    if !info.funds.is_empty() {
        return Err(ContractError::NoFundsShouldBeAttached);
    }
    let config: Config = CONFIG.load(deps.storage)?;
    if config.paused {
        return Err(ContractError::ContractPaused);
    }

    let agent_id = info.sender;

    // REF: https://github.com/CosmWasm/cw-tokens/tree/main/contracts/cw20-escrow
    // Check if native token balance is sufficient for a few txns, in this case 4 txns
    let agent_wallet_balances = deps.querier.query_all_balances(agent_id.clone())?;

    // Get the denom from the manager contract
    let manager_config = croncat_manager_contract::query_manager_config(&deps.as_ref(), &config)?;

    let agents_needs_coin = Coin::new(
        config.min_coins_for_agent_registration.into(),
        manager_config.native_denom,
    );
    if !has_coins(&agent_wallet_balances, &agents_needs_coin) || agent_wallet_balances.is_empty() {
        return Err(ContractError::InsufficientFunds {
            amount_needed: agents_needs_coin,
        });
    }

    let payable_account_id = if let Some(addr) = payable_account_id {
        deps.api.addr_validate(&addr)?
    } else {
        agent_id.clone()
    };

    let (_, agent) =
        AGENT_DISTRIBUTOR.add_new_agent(deps.storage, &env, agent_id, payable_account_id)?;
    Ok(Response::new()
        .add_attribute("action", "register_agent")
        .add_attribute("agent_status", agent.status.to_string()))
}

/// Update agent details, specifically the payable account id for an agent.
fn update_agent(
    deps: DepsMut,
    info: MessageInfo,
    payable_account_id: String,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    if config.paused {
        return Err(ContractError::ContractPaused);
    }
    let payable_account_id = deps.api.addr_validate(&payable_account_id)?;
    AGENT_DISTRIBUTOR.set_payable_account_id(deps.storage, info.sender, payable_account_id)?;

    Ok(Response::new().add_attribute("action", "update_agent"))
}

/// Allows an agent to accept a nomination within a certain amount of time to become an active agent.
fn accept_nomination_agent(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
) -> Result<Response, ContractError> {
    // Compare current time and Config's agent_nomination_begin_time to see if agent can join
    let config: Config = CONFIG.load(deps.storage)?;
    if config.paused {
        return Err(ContractError::ContractPaused);
    }
    AGENT_DISTRIBUTOR.try_nominate_agent(deps.storage, &env, &config, info.sender.clone())?;

    // Find difference
    Ok(Response::new()
        .add_attribute("action", "accept_nomination_agent")
        .add_attribute("new_agent", info.sender.as_str()))
}

/// Removes the agent from the active set of AGENTS.
/// Withdraws all reward balances to the agent payable account id.
/// In case it fails to unregister pending agent try to set `from_behind` to true
fn unregister_agent(
    storage: &mut dyn Storage,
    querier: &QuerierWrapper<Empty>,
    agent_id: &Addr,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(storage)?;
    if config.paused {
        return Err(ContractError::ContractPaused);
    }
    let agent = AGENT_DISTRIBUTOR
        .get_agent(storage, agent_id)?
        .ok_or(ContractError::AgentNotRegistered)?;

    let msg = croncat_manager_contract::create_withdraw_rewards_submsg(
        querier,
        &config,
        agent_id.as_str(),
        agent.payable_account_id.to_string(),
    )?;
    AGENT_DISTRIBUTOR.remove_agent(storage, agent_id)?;
    let responses = Response::new()
        //Send withdraw rewards message to manager contract
        .add_message(msg)
        .add_attribute("action", "unregister_agent")
        .add_attribute("account_id", agent_id);

    Ok(responses)
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    msg: UpdateConfig,
) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |config| {
        // Deconstruct, so we don't miss any fields
        let UpdateConfig {
            owner_addr,
            paused,
            croncat_factory_addr,
            croncat_manager_key,
            croncat_tasks_key,
            min_tasks_per_agent,
            agent_nomination_duration,
            min_coins_for_agent_registration,
            max_slot_passover,
            min_active_reserve,
        } = msg;

        if info.sender != config.owner_addr {
            return Err(ContractError::Unauthorized {});
        }

        let new_config = Config {
            owner_addr: owner_addr
                .map(|human| deps.api.addr_validate(&human))
                .transpose()?
                .unwrap_or(config.owner_addr),
            croncat_factory_addr: croncat_factory_addr
                .map(|human| deps.api.addr_validate(&human))
                .transpose()?
                .unwrap_or(config.croncat_factory_addr),
            croncat_manager_key: croncat_manager_key.unwrap_or(config.croncat_manager_key),
            croncat_tasks_key: croncat_tasks_key.unwrap_or(config.croncat_tasks_key),
            paused: paused.unwrap_or(config.paused),
            min_tasks_per_agent: min_tasks_per_agent.unwrap_or(config.min_tasks_per_agent),
            agent_nomination_block_duration: agent_nomination_duration
                .unwrap_or(config.agent_nomination_block_duration),
            min_coins_for_agent_registration: min_coins_for_agent_registration
                .unwrap_or(DEFAULT_MIN_COINS_FOR_AGENT_REGISTRATION),
            max_slot_passover: max_slot_passover.unwrap_or(DEFAULT_MAX_SLOTS_PASSOVER),
            min_active_reserve: min_active_reserve.unwrap_or(DEFAULT_MIN_ACTIVE_RESERVE),
        };
        Ok(new_config)
    })?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn execute_tick(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut attributes = vec![];
    let mut submessages = vec![];
    let agents_to_delete = AGENT_DISTRIBUTOR.cleanup(deps.storage, &env, &config)?;
    for agent_id in agents_to_delete {
        let resp = unregister_agent(deps.storage, &deps.querier, &agent_id).unwrap();
        // Save attributes and messages
        attributes.extend_from_slice(&resp.attributes);
        submessages.extend_from_slice(&resp.messages);
    }
    // Check if there isn't any active or pending agents
    if !(AGENT_DISTRIBUTOR.has_active(deps.storage)?
        && AGENT_DISTRIBUTOR.has_pending(deps.storage)?)
    {
        attributes.push(Attribute::new("lifecycle", "tick_failure"))
    }
    let response = Response::new()
        .add_attribute("action", "tick")
        .add_attributes(attributes)
        .add_submessages(submessages);
    Ok(response)
}

fn on_task_created(
    env: Env,
    deps: DepsMut,
    info: MessageInfo,
    _: AgentOnTaskCreated,
) -> Result<Response, ContractError> {
    let config = CONFIG.may_load(deps.storage)?.unwrap();
    croncat_tasks_contract::assert_caller_is_tasks_contract(&deps.querier, &config, &info.sender)?;

    AGENT_DISTRIBUTOR.notify_task_created(deps.storage, &env, &config, None)?;
    let response = Response::new().add_attribute("action", "on_task_created");
    Ok(response)
}
fn on_task_completed(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    args: AgentOnTaskCompleted,
) -> Result<Response, ContractError> {
    let config = CONFIG.may_load(deps.storage)?.unwrap();

    croncat_manager_contract::assert_caller_is_manager_contract(
        &deps.querier,
        &config,
        &info.sender,
    )?;

    AGENT_DISTRIBUTOR.notify_task_completed(
        deps.storage,
        env,
        args.agent_id,
        args.is_block_slot_task,
    )?;
    let response = Response::new().add_attribute("action", "on_task_completed");
    Ok(response)
}
