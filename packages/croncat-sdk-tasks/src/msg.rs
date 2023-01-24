use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::types::TaskRequest;

#[cw_serde]
pub struct TasksInstantiateMsg {
    /// Address of the factory contract
    pub croncat_factory_addr: String,

    /// Chain name to add prefix to the task_hash
    pub chain_name: String,

    /// Address of the contract owner, defaults to the sender
    pub owner_addr: Option<String>,
    /// Name of the key for raw querying Manager address from the factory
    pub croncat_manager_key: (String, [u8; 2]),
    /// Name of the key for raw querying Agents address from the factory
    pub croncat_agents_key: (String, [u8; 2]),

    /// Time in nanos for each bucket of tasks
    pub slot_granularity_time: Option<u64>,

    /// Gas needed to cover proxy call without any action
    pub gas_base_fee: Option<u64>,
    /// Gas needed to cover single non-wasm task's Action
    pub gas_action_fee: Option<u64>,
    /// Gas needed to cover single query
    pub gas_query_fee: Option<u64>,

    /// Gas limit, to make sure task won't lock contract
    pub gas_limit: Option<u64>,
}

#[cw_serde]
pub enum TasksExecuteMsg {
    CreateTask { task: TaskRequest },
    RemoveTask { task_hash: String },
    // Methods for other contracts
    RemoveTaskInternal { task_hash: Vec<u8> },
    TryToRescheduleTask { task_hash: Vec<u8> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum TasksQueryMsg {
    #[returns(Vec<crate::types::TaskResponse>)]
    Tasks {
        from_index: Option<u64>,
        limit: Option<u64>,
    },
    #[returns(Vec<crate::types::TaskResponse>)]
    TasksWithQueries {
        from_index: Option<u64>,
        limit: Option<u64>,
    },
    #[returns(Vec<crate::types::TaskResponse>)]
    TasksByOwner {
        owner_addr: String,
        from_index: Option<u64>,
        limit: Option<u64>,
    },
    #[returns(Option<crate::types::TaskResponse>)]
    Task { task_hash: String },
    #[returns(String)]
    TaskHash { task: Box<crate::types::Task> },
    #[returns(crate::types::SlotHashesResponse)]
    SlotHashes { slot: Option<u64> },
    #[returns(crate::types::SlotIdsResponse)]
    SlotIds {
        from_index: Option<u64>,
        limit: Option<u64>,
    },
    #[returns(Option<crate::types::CurrentTaskResponse>)]
    GetCurrentTask {},
}
