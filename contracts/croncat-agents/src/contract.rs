use std::cmp;
use std::ops::Div;

use crate::balancer::Balancer;
use crate::error::ContractError;
use crate::msg::{
    AgentContractInstantiateMsg as InstantiateMsg, 
    AgentExecuteMsg as ExecuteMsg,
    AgentQueryMsg as QueryMsg,
};
use cw2::set_contract_version;

use crate::state::{
    ACTIVE_AGENTS, AGENTS, AGENT_BALANCER, AGENT_NOMINATION_BEGIN_TIME, CONFIG, PENDING_AGENTS,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    has_coins, to_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Storage, SubMsg, Uint128,
};
use croncat_sdk_agents::msg::{AgentResponse, AgentTaskResponse, GetAgentIdsResponse};
use croncat_sdk_agents::types::{Agent, AgentStatus};
use croncat_sdk_core::types::Config;

pub(crate) const CONTRACT_NAME: &str = "crates.io:croncat-agents";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("paused", config.paused.to_string())
        .add_attribute("owner_id", config.owner_id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAgent {
            account_id,
            total_tasks,
        } => to_binary(&query_get_agent(deps, env, account_id, total_tasks)?),
        QueryMsg::GetAgentIds {} => to_binary(&query_get_agent_ids(deps)?),
        QueryMsg::GetAgentTasks { account_id, slots } => {
            to_binary(&query_get_agent_tasks(deps, env, account_id, slots)?)
        }
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
        ExecuteMsg::RegisterAgent {
            payable_account_id,
            cost,
        } => register_agent(deps, info, env, payable_account_id, cost),
        ExecuteMsg::UpdateAgent { payable_account_id } => {
            update_agent(deps, info, env, payable_account_id)
        }
        ExecuteMsg::UnregisterAgent { from_behind } => {
            unregister_agent(deps.storage, &info.sender, from_behind)
        }
        ExecuteMsg::WithdrawReward {} => withdraw_agent_balance(deps, &info.sender),
        ExecuteMsg::CheckInAgent {} => accept_nomination_agent(deps, info, env),
    }
}

fn query_get_agent(
    deps: Deps,
    env: Env,
    account_id: String,
    total_tasks: u64,
) -> StdResult<Option<AgentResponse>> {
    let account_id = deps.api.addr_validate(&account_id)?;
    let agent = AGENTS.may_load(deps.storage, &account_id)?;
    let a = if let Some(a) = agent {
        a
    } else {
        return Ok(None);
    };

    let agent_status = get_agent_status(deps.storage, env, account_id, total_tasks)
        // Return wrapped error if there was a problem
        .map_err(|err| StdError::GenericErr {
            msg: err.to_string(),
        })?;

    let agent_response = AgentResponse {
        status: agent_status,
        payable_account_id: a.payable_account_id,
        balance: a.balance,
        total_tasks_executed: a.total_tasks_executed,
        last_executed_slot: a.last_executed_slot,
        register_start: a.register_start,
    };
    Ok(Some(agent_response))
}

/// Get a list of agent addresses
fn query_get_agent_ids(deps: Deps) -> StdResult<GetAgentIdsResponse> {
    let active: Vec<Addr> = ACTIVE_AGENTS.load(deps.storage)?;
    let pending: Vec<Addr> = PENDING_AGENTS
        .iter(deps.storage)?
        .collect::<StdResult<Vec<Addr>>>()?;

    Ok(GetAgentIdsResponse { active, pending })
}

fn query_get_agent_tasks(
    deps: Deps,
    env: Env,
    account_id: String,
    slots: (u64, u64), //block_slots,cron_slots
) -> StdResult<Option<AgentTaskResponse>> {
    let account_id = deps.api.addr_validate(&account_id)?;
    let active = ACTIVE_AGENTS.load(deps.storage)?;
    if !active.contains(&account_id) {
        return Err(StdError::GenericErr {
            msg: "Agent is not active!".to_owned(),
        });
    }

    if slots == (0, 0) {
        return Ok(None);
    }
    AGENT_BALANCER
        .get_agent_tasks(&deps, &env, &CONFIG, &ACTIVE_AGENTS, account_id, slots)
        .map_err(|err| StdError::generic_err(err.to_string()))
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
    cost: u128,
) -> Result<Response, ContractError> {
    if !info.funds.is_empty() {
        return Err(ContractError::InsufficientFunds);
    }
    let c: Config = CONFIG.load(deps.storage)?;
    if c.paused {
        return Err(ContractError::ContractPaused);
    }

    let account = info.sender;

    // REF: https://github.com/CosmWasm/cw-tokens/tree/main/contracts/cw20-escrow
    // Check if native token balance is sufficient for a few txns, in this case 4 txns
    let agent_wallet_balances = deps.querier.query_all_balances(account.clone())?;
    if !has_coins(&agent_wallet_balances, &Coin::new(cost, c.native_denom))
        || agent_wallet_balances.is_empty()
    {
        return Err(ContractError::InsufficientFunds);
    }

    let payable_id = if let Some(addr) = payable_account_id {
        deps.api.addr_validate(&addr)?
    } else {
        account.clone()
    };

    let mut active_agents_vec: Vec<Addr> = ACTIVE_AGENTS.load(deps.storage)?;
    let total_agents = active_agents_vec.len();
    let agent_status = if total_agents == 0 {
        active_agents_vec.push(account.clone());
        ACTIVE_AGENTS.save(deps.storage, &active_agents_vec)?;
        AgentStatus::Active
    } else {
        PENDING_AGENTS.push_back(deps.storage, &account)?;
        AgentStatus::Pending
    };
    let agent = AGENTS.update(
        deps.storage,
        &account,
        |a: Option<Agent>| -> Result<_, ContractError> {
            match a {
                // make sure that account isn't already added
                Some(_) => Err(ContractError::AgentAlreadyRegistered),
                None => {
                    Ok(Agent {
                        payable_account_id: payable_id,
                        balance: Uint128::default(),
                        total_tasks_executed: 0,
                        last_executed_slot: env.block.height,
                        // REF: https://github.com/CosmWasm/cosmwasm/blob/main/packages/std/src/types.rs#L57
                        register_start: env.block.time,
                    })
                }
            }
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "register_agent")
        .add_attribute("agent_status", format!("{:?}", agent_status))
        .add_attribute("register_start", agent.register_start.nanos().to_string())
        .add_attribute("payable_account_id", agent.payable_account_id))
}

/// Update agent details, specifically the payable account id for an agent.
fn update_agent(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    payable_account_id: String,
) -> Result<Response, ContractError> {
    let payable_account_id = deps.api.addr_validate(&payable_account_id)?;
    let c: Config = CONFIG.load(deps.storage)?;
    if c.paused {
        return Err(ContractError::ContractPaused);
    }

    let agent = AGENTS.update(
        deps.storage,
        &info.sender,
        |a: Option<Agent>| -> Result<_, ContractError> {
            match a {
                Some(agent) => {
                    let mut ag = agent;
                    ag.payable_account_id = payable_account_id;
                    Ok(ag)
                }
                None => Err(ContractError::AgentNotRegistered {}),
            }
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "update_agent")
        .add_attribute("payable_account_id", agent.payable_account_id))
}

/// Allows an agent to withdraw all rewards, paid to the specified payable account id.
fn withdraw_rewards(
    storage: &mut dyn Storage,
    agent_id: &Addr,
) -> Result<Vec<SubMsg>, ContractError> {
    let mut agent = AGENTS
        .may_load(storage, agent_id)?
        .ok_or(ContractError::AgentNotRegistered {})?;

    // This will send all token balances to Agent
    let (messages, balance) = send_tokens(storage, &agent.payable_account_id, agent.balance)?;
    agent.balance -= balance;
    AGENTS.save(storage, agent_id, &agent)?;

    Ok(messages)
}

// Helper to distribute funds/tokens
fn send_tokens(
    storage: &mut dyn Storage,
    to: &Addr,
    balance: Uint128,
) -> StdResult<(Vec<SubMsg>, Uint128)> {
    let config = CONFIG.may_load(storage)?.unwrap();
    let msgs: Vec<SubMsg> = if balance.is_zero() {
        vec![]
    } else {
        vec![SubMsg::new(BankMsg::Send {
            to_address: to.into(),
            amount: vec![Coin {
                denom: config.native_denom,
                amount: balance,
            }],
        })]
    };
    Ok((msgs, balance))
}
/// Allows an agent to withdraw all rewards, paid to the specified payable account id.
fn withdraw_agent_balance(deps: DepsMut, agent_id: &Addr) -> Result<Response, ContractError> {
    let messages = withdraw_rewards(deps.storage, agent_id)?;

    Ok(Response::new()
        .add_attribute("method", "withdraw_agent_balance")
        .add_attribute("account_id", agent_id)
        .add_submessages(messages))
}

/// Allows an agent to accept a nomination within a certain amount of time to become an active agent.
fn accept_nomination_agent(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
) -> Result<Response, ContractError> {
    // Compare current time and Config's agent_nomination_begin_time to see if agent can join
    let c: Config = CONFIG.load(deps.storage)?;

    let mut active_agents: Vec<Addr> = ACTIVE_AGENTS.load(deps.storage)?;
    let mut pending_queue_iter = PENDING_AGENTS.iter(deps.storage)?;
    // Agent must be in the pending queue
    // Get the position in the pending queue
    let agent_position = if let Some(agent_position) = pending_queue_iter.position(|address| {
        if let Ok(addr) = address {
            addr == info.sender
        } else {
            false
        }
    }) {
        agent_position
    } else {
        // Sender's address does not exist in the agent pending queue
        return Err(ContractError::AgentNotRegistered);
    };
    let time_difference =
        if let Some(nomination_start) = AGENT_NOMINATION_BEGIN_TIME.load(deps.storage)? {
            env.block.time.seconds() - nomination_start.seconds()
        } else {
            // edge case if last agent left
            if active_agents.is_empty() && agent_position == 0 {
                active_agents.push(info.sender.clone());
                ACTIVE_AGENTS.save(deps.storage, &active_agents)?;

                PENDING_AGENTS.pop_front(deps.storage)?;
                AGENT_NOMINATION_BEGIN_TIME.save(deps.storage, &None)?;
                return Ok(Response::new()
                    .add_attribute("method", "accept_nomination_agent")
                    .add_attribute("new_agent", info.sender.as_str()));
            } else {
                // No agents can join yet
                return Err(ContractError::NotAcceptingNewAgents);
            }
        };

    // It works out such that the time difference between when this is called,
    // and the agent nomination begin time can be divided by the nomination
    // duration and we get an integer. We use that integer to determine if an
    // agent is allowed to get let in. If their position in the pending queue is
    // less than or equal to that integer, they get let in.
    let max_index = time_difference.div(c.agent_nomination_duration as u64);
    let kicked_agents = if agent_position as u64 <= max_index {
        // Make this agent active
        // Update state removing from pending queue
        let kicked_agents: Vec<Addr> = {
            let mut kicked = Vec::with_capacity(agent_position);
            for _ in 0..=agent_position {
                let agent = PENDING_AGENTS.pop_front(deps.storage)?;
                // Since we already iterated over it - we know it exists
                let kicked_agent;
                unsafe {
                    kicked_agent = agent.unwrap_unchecked();
                }
                kicked.push(kicked_agent);
            }
            kicked
        };

        // and adding to active queue
        active_agents.push(info.sender.clone());
        ACTIVE_AGENTS.save(deps.storage, &active_agents)?;

        // and update the config, setting the nomination begin time to None,
        // which indicates no one will be nominated until more tasks arrive
        AGENT_NOMINATION_BEGIN_TIME.save(deps.storage, &None)?;
        kicked_agents
    } else {
        return Err(ContractError::TryLaterForNomination);
    };
    // Find difference
    Ok(Response::new()
        .add_attribute("method", "accept_nomination_agent")
        .add_attribute("new_agent", info.sender.as_str())
        .add_attribute("kicked_agents: ", format!("{kicked_agents:?}")))
}

/// Removes the agent from the active set of AGENTS.
/// Withdraws all reward balances to the agent payable account id.
/// In case it fails to unregister pending agent try to set `from_behind` to true
fn unregister_agent(
    storage: &mut dyn Storage,
    agent_id: &Addr,
    from_behind: Option<bool>,
) -> Result<Response, ContractError> {
    // Get withdraw messages, if any
    // NOTE: Since this also checks if agent exists, safe to not have redundant logic
    let messages = withdraw_rewards(storage, agent_id)?;
    AGENTS.remove(storage, agent_id);

    // Remove from the list of active agents if the agent in this list
    let mut active_agents: Vec<Addr> = ACTIVE_AGENTS.load(storage)?;
    if let Some(index) = active_agents.iter().position(|addr| addr == agent_id) {
        //Notify the balancer agent has been removed, to rebalance itself
        AGENT_BALANCER.on_agent_unregister(storage, &ACTIVE_AGENTS, agent_id.clone())?;
        active_agents.remove(index);

        ACTIVE_AGENTS.save(storage, &active_agents)?;
    } else {
        // Agent can't be both in active and pending vector
        // Remove from the pending queue
        let mut return_those_agents: Vec<Addr> =
            Vec::with_capacity((PENDING_AGENTS.len(storage)? / 2) as usize);
        if from_behind.unwrap_or(false) {
            while let Some(addr) = PENDING_AGENTS.pop_front(storage)? {
                if addr.eq(agent_id) {
                    break;
                } else {
                    return_those_agents.push(addr);
                }
            }
            for ag in return_those_agents.iter().rev() {
                PENDING_AGENTS.push_front(storage, ag)?;
            }
        } else {
            while let Some(addr) = PENDING_AGENTS.pop_back(storage)? {
                if addr.eq(agent_id) {
                    break;
                } else {
                    return_those_agents.push(addr);
                }
            }
            for ag in return_those_agents.iter().rev() {
                PENDING_AGENTS.push_back(storage, ag)?;
            }
        }
    }

    let responses = Response::new()
        .add_attribute("method", "unregister_agent")
        .add_attribute("account_id", agent_id);

    if messages.is_empty() {
        Ok(responses)
    } else {
        Ok(responses.add_submessages(messages))
    }
}

fn get_agent_status(
    storage: &dyn Storage,
    env: Env,
    account_id: Addr,
    total_tasks: u64,
) -> Result<AgentStatus, ContractError> {
    let c: Config = CONFIG.load(storage)?;
    let active = ACTIVE_AGENTS.load(storage)?;

    // Pending
    let mut pending_iter = PENDING_AGENTS.iter(storage)?;
    // If agent is pending, Check if they should get nominated to checkin to become active
    let agent_position = if let Some(pos) = pending_iter.position(|address| {
        if let Ok(addr) = address {
            addr == account_id
        } else {
            false
        }
    }) {
        pos
    } else {
        // Check for active
        if active.contains(&account_id) {
            return Ok(AgentStatus::Active);
        } else {
            return Err(ContractError::AgentNotRegistered {});
        }
    };

    // Edge case if last agent unregistered
    if active.is_empty() && agent_position == 0 {
        return Ok(AgentStatus::Nominated);
    };

    // Load config's task ratio, total tasks, active agents, and AGENT_NOMINATION_BEGIN_TIME.
    // Then determine if this agent is considered "Nominated" and should call CheckInAgent
    let max_agent_index =
        max_agent_nomination_index(storage, &c, env, &(active.len() as u64), total_tasks)?;
    let agent_status = match max_agent_index {
        Some(max_idx) if agent_position as u64 <= max_idx => AgentStatus::Nominated,
        _ => AgentStatus::Pending,
    };
    Ok(agent_status)
}

/// Calculate the biggest index of nomination for pending agents
fn max_agent_nomination_index(
    storage: &dyn Storage,
    cfg: &Config,
    env: Env,
    num_active_agents: &u64,
    total_tasks: u64,
) -> Result<Option<u64>, ContractError> {
    let block_time = env.block.time.seconds();

    let agent_nomination_begin_time = AGENT_NOMINATION_BEGIN_TIME.load(storage)?;

    match agent_nomination_begin_time {
        Some(begin_time) => {
            let min_tasks_per_agent = cfg.min_tasks_per_agent;
            let num_agents_to_accept =
                agents_to_let_in(&min_tasks_per_agent, num_active_agents, &total_tasks);

            if num_agents_to_accept > 0 {
                let time_difference = block_time - begin_time.seconds();

                let max_index = cmp::max(
                    time_difference.div(cfg.agent_nomination_duration as u64),
                    num_agents_to_accept - 1,
                );
                Ok(Some(max_index))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

fn agents_to_let_in(max_tasks: &u64, num_active_agents: &u64, total_tasks: &u64) -> u64 {
    let num_tasks_covered = num_active_agents * max_tasks;
    if total_tasks > &num_tasks_covered {
        // It's possible there are more "covered tasks" than total tasks,
        // so use saturating subtraction to hit zero and not go below
        let total_tasks_needing_agents = total_tasks.saturating_sub(num_tasks_covered);
        let remainder = u64::from(total_tasks_needing_agents % max_tasks != 0);
        total_tasks_needing_agents / max_tasks + remainder
    } else {
        0
    }
}
