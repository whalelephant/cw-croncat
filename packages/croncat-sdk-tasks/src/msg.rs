use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::types::TaskRequest;

#[cw_serde]
pub struct TasksInstantiateMsg {
    pub manager_addr: String,
}

#[cw_serde]
pub enum TasksExecuteMsg {
    CreateTask{
        task: TaskRequest
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum TasksQueryMsg {}
