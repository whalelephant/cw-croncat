use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128, Uint64};

use crate::types::AgentStatus;

#[cw_serde]
pub enum AgentExecuteMsg {
    RegisterAgent { payable_account_id: Option<String> },
    UpdateAgent { payable_account_id: String },
    CheckInAgent {},
    UnregisterAgent { from_behind: Option<bool> },
    WithdrawReward {},
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
    pub num_block_tasks_extra: Uint64,
    pub num_cron_tasks: Uint64,
    pub num_cron_tasks_extra: Uint64,
}
