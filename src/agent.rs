use crate::error::ContractError;
use crate::state::{
    Agent, AgentStatus, Config, GenericBalance, AGENTS, AGENTS_ACTIVE_QUEUE, AGENTS_PENDING_QUEUE,
    CONFIG,
};
use cosmwasm_std::{has_coins, Addr, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};
use cw20::Balance;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetAgentResponse {
    pub status: AgentStatus,
    pub payable_account_id: Addr,
    pub balance: GenericBalance,
    pub total_tasks_executed: u64,
    pub last_missed_slot: u64,
    pub register_start: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetAgentIdsResponse(Vec<Addr>, Vec<Addr>);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetAgentTasksResponse(u64, u128);

/// Get a single agent details
/// Check's status as well, in case this agent needs to be considered for election
pub(crate) fn query_get_agent(deps: Deps, account_id: Addr) -> StdResult<Option<GetAgentResponse>> {
    let agent = AGENTS.may_load(deps.storage, account_id.clone())?;
    if agent.is_none() { return Ok(None) }
    let a = agent.unwrap();

    let pending = AGENTS_PENDING_QUEUE.load(deps.storage)?;

    // If agent is pending, Check if they should get nominated to checkin to become active
    let agent_status: AgentStatus = if a.status == AgentStatus::Pending {
      // TODO: change to check total tasks + task ratio
      if pending.contains(&account_id) {
        AgentStatus::Nominated
      } else {
        a.status
      }
    } else {
      a.status
    };

    Ok(Some(GetAgentResponse {
        status: agent_status,
        payable_account_id: a.payable_account_id,
        balance: a.balance,
        total_tasks_executed: a.total_tasks_executed,
        last_missed_slot: a.last_missed_slot,
        register_start: a.register_start,
    }))
}

/// Add any account as an agent that will be able to execute tasks.
/// Registering allows for rewards accruing with micro-payments which will accumulate to more long-term.
///
/// Optional Parameters:
/// "payable_account_id" - Allows a different account id to be specified, so a user can receive funds at a different account than the agent account.
// TODO: Finish Transition Logic!
pub fn register_agent(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    payable_account_id: Option<Addr>,
) -> Result<Response, ContractError> {
    let c: Config = CONFIG.load(deps.storage)?;
    if c.paused {
        return Err(ContractError::CustomError {
            val: "Register agent paused".to_string(),
        });
    }

    // REF: https://github.com/CosmWasm/cw-tokens/tree/main/contracts/cw20-escrow
    if has_coins(info.funds.as_ref(), &c.agent_storage_deposit) {
        return Err(ContractError::CustomError {
            val: "Insufficient deposit".to_string(),
        });
    }
    let deposit: Balance = Balance::from(info.funds);
    if deposit.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }

    let account = info.sender.clone();
    let payable_id = payable_account_id.unwrap_or_else(|| account.clone());

    let push_account = |mut aq: Vec<Addr>| -> Result<_, ContractError> {
        aq.push(account.clone());
        Ok(aq)
    };

    let active_agents = AGENTS_ACTIVE_QUEUE.load(deps.storage)?;
    let total_agents = active_agents.len();
    let agent_status = if total_agents == 0 {
        AGENTS_ACTIVE_QUEUE.update(deps.storage, push_account)?;
        AgentStatus::Active
    } else {
        AGENTS_PENDING_QUEUE.update(deps.storage, push_account)?;
        AgentStatus::Pending
    };

    AGENTS.update(
        deps.storage,
        account,
        |a: Option<Agent>| -> Result<_, ContractError> {
            match a {
                Some(_) => {
                    Ok(Agent {
                        status: agent_status.clone(),
                        payable_account_id: payable_id,
                        balance: GenericBalance::default(),
                        total_tasks_executed: 0,
                        last_missed_slot: 0,
                        // REF: https://github.com/CosmWasm/cosmwasm/blob/main/packages/std/src/types.rs#L57
                        register_start: env.block.time.nanos(),
                    })
                }
                // make sure that account isn't already added
                None => Err(ContractError::CustomError {
                    val: "Agent already exists".to_string(),
                }),
            }
        },
    )?;

    // update balances
    CONFIG.update(deps.storage, |mut config| -> Result<_, StdError> {
        config.available_balance.add_tokens(deposit);
        Ok(config)
    })?;

    Ok(Response::new()
        .add_attribute("method", "register_agent")
        .add_attribute("agent_status", format!("{:?}", agent_status)))
}
