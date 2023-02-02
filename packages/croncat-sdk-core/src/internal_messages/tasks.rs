use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Binary, CosmosMsg, StdResult, WasmMsg};

#[cw_serde]
pub struct TasksRemoveTaskByManager {
    pub task_hash: Vec<u8>,
}

impl TasksRemoveTaskByManager {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = RemoveTaskByManager::RemoveTaskByManager(self);
        to_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>>(self, contract_addr: T) -> StdResult<CosmosMsg> {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}

#[cw_serde]
pub(crate) enum RemoveTaskByManager {
    RemoveTaskByManager(TasksRemoveTaskByManager),
}

#[cw_serde]
pub struct TasksRescheduleTask {
    pub task_hash: Vec<u8>,
}

impl TasksRescheduleTask {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = RescheduleTaskMsg::RescheduleTask(self);
        to_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>>(self, contract_addr: T) -> StdResult<CosmosMsg> {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}

#[cw_serde]
pub(crate) enum RescheduleTaskMsg {
    RescheduleTask(TasksRescheduleTask),
}
