use crate::types::AgentStatus;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Timestamp, Uint128, Uint64};
use croncat_sdk_core::internal_messages::agents::{AgentOnTaskCompleted, AgentOnTaskCreated};

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of the contract owner, defaults to the sender
    pub owner_addr: Option<String>,
    /// CW2 Version provided by factory
    pub version: Option<String>,
    /// Name of the key for raw querying Manager address from the factory
    pub croncat_manager_key: (String, [u8; 2]),
    /// Name of the key for raw querying Tasks address from the factory
    pub croncat_tasks_key: (String, [u8; 2]),

    /// Sets the amount of time opportunity for a pending agent to become active.
    /// If there is a pending queue, the longer a pending agent waits,
    /// the more pending agents can potentially become active based on this nomination window.
    /// This duration doesn't block the already nominated agent from becoming active,
    /// it only opens the door for more to become active. If a pending agent is nominated,
    /// then is lazy and beat by another agent, they get removed from pending queue and must
    /// register again.
    pub agent_nomination_duration: Option<u16>,
    /// The ratio used to calculate active agents/tasks. Example: "3", requires there are
    /// 4 tasks before letting in another agent to become active. (3 tasks for agent 1, 1 task for agent 2)
    pub min_tasks_per_agent: Option<u64>,
    /// The required amount needed to actually execute a few tasks before withdraw profits.
    /// This helps make sure agent wont get stuck out the gate
    pub min_coin_for_agent_registration: Option<u64>,

    /// How many slots an agent can miss before being removed from the active queue
    pub agents_eject_threshold: Option<u64>,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterAgent { payable_account_id: Option<String> },
    UpdateAgent { payable_account_id: String },
    CheckInAgent {},
    UnregisterAgent { from_behind: Option<bool> },
    //Task contract will send message when task is created
    OnTaskCreated(AgentOnTaskCreated),
    OnTaskCompleted(AgentOnTaskCompleted),
    UpdateConfig { config: UpdateConfig },
    //Tick action will remove unactive agents periodically
    Tick {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns[AgentResponse]]
    GetAgent { account_id: String },
    #[returns[GetAgentIdsResponse]]
    GetAgentIds {
        from_index: Option<u64>,
        limit: Option<u64>,
    },
    #[returns[AgentTaskResponse]]
    GetAgentTasks { account_id: String },
    #[returns[crate::types::Config]]
    Config {},
}

#[cw_serde]
pub struct GetAgentIdsResponse {
    pub active: Vec<Addr>,
    pub pending: Vec<Addr>,
}
#[cw_serde]
pub struct AgentInfo {
    pub status: AgentStatus,
    pub payable_account_id: Addr,
    pub balance: Uint128,
    pub last_executed_slot: u64,
    pub register_start: Timestamp,
}
#[cw_serde]
pub struct AgentResponse {
    pub agent: Option<AgentInfo>,
}
#[cw_serde]
pub struct TaskStats {
    pub num_block_tasks: Uint64,
    pub num_cron_tasks: Uint64,
}
#[cw_serde]
pub struct AgentTaskResponse {
    pub stats: TaskStats,
}

#[cw_serde]
pub struct UpdateConfig {
    pub owner_addr: Option<String>,
    pub paused: Option<bool>,
    /// Address of the factory contract
    pub croncat_factory_addr: Option<String>,
    /// Name of the key for raw querying Manager address from the factory
    pub croncat_manager_key: Option<(String, [u8; 2])>,
    /// Name of the key for raw querying Tasks address from the factory
    pub croncat_tasks_key: Option<(String, [u8; 2])>,

    pub min_tasks_per_agent: Option<u64>,
    pub agent_nomination_duration: Option<u16>,
    pub min_coins_for_agent_registration: Option<u64>,
    // How many slots an agent can miss before being removed from the active queue
    pub agents_eject_threshold: Option<u64>,
}
