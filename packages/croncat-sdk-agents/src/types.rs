use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};

#[cw_serde]
pub enum AgentExecuteMsg {
    RegisterAgent { payable_account_id: Option<String> },
    UpdateAgent { payable_account_id: String },
    CheckInAgent {},
    UnregisterAgent { from_behind: Option<bool> },
    WithdrawReward {},
}

#[cw_serde]
pub enum AgentStatus {
    // Default for any new agent, if tasks ratio allows
    Active,

    // Default for any new agent, until more tasks come online
    Pending,

    // More tasks are available, agent must checkin to become active
    Nominated,
}

#[cw_serde]
pub struct Agent {
    // Where rewards get transferred
    pub payable_account_id: Addr,

    // accrued reward balance
    pub balance: Uint128,

    // Timestamp of when agent first registered
    // Useful for rewarding agents for their patience while they are pending and operating service
    // Agent will be responsible to constantly monitor when it is their turn to join in active agent set (done as part of agent code loops)
    // Example data: 1633890060000000000 or 0
    pub register_start: Timestamp,
}

#[cw_serde]
#[derive(Default)]
pub struct AgentStats {
    pub completed_block_tasks: u64,
    pub completed_cron_tasks: u64,
    pub missed_blocked_tasks: u64,
    pub missed_cron_tasks: u64,
    // Holds slot number of the last slot when agent called proxy_call.
    // If agent does a task, this number is set to the current block.
    pub last_executed_slot: u64,
}
