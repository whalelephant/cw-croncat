use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Timestamp, Uint128, Uint64};

use crate::types::AgentStatus;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner_addr: Option<String>,
    pub native_denom: Option<String>,
    pub agent_nomination_duration: Option<u16>,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterAgent {
        payable_account_id: Option<String>,
        cost: u128,
    },
    UpdateAgent {
        payable_account_id: String,
    },
    CheckInAgent {},
    UnregisterAgent {
        from_behind: Option<bool>,
    },
    WithdrawReward {},
    NewTaskCreated {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns[Option<AgentResponse>]]
    GetAgent {
        account_id: String,
        total_tasks: u64,
    },
    #[returns[Option<GetAgentIdsResponse>]]
    GetAgentIds {},
    #[returns[Option<AgentTaskResponse>]]
    GetAgentTasks {
        account_id: String,
        slots: (Option<u64>, Option<u64>),
    },
}

#[cw_serde]
pub struct GetAgentIdsResponse {
    pub active: Vec<Addr>,
    pub pending: Vec<Addr>,
}

#[cw_serde]
pub struct AgentResponse {
    pub status: AgentStatus,
    pub payable_account_id: Addr,
    pub balance: Uint128,
    pub total_tasks_executed: u64,
    pub last_executed_slot: u64,
    pub register_start: Timestamp,
}

#[cw_serde]
pub struct AgentTaskResponse {
    pub num_block_tasks: Uint64,
    pub num_cron_tasks: Uint64,
}

#[cw_serde]
pub struct Config {
    pub owner_addr: Addr,
    pub paused: bool,
    pub native_denom: String,
    pub min_tasks_per_agent: u64,
    /// The duration a prospective agent has to nominate themselves.
    /// When a task is created such that a new agent can join,
    /// The agent at the zeroth index of the pending agent queue has this time to nominate
    /// The agent at the first index has twice this time to nominate (which would remove the former agent from the pending queue)
    /// Value is in seconds
    pub agent_nomination_duration: u16,
}
