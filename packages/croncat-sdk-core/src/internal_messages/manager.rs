use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Binary, CosmosMsg, StdResult, WasmMsg, Coin};

use crate::types::AmountForOneTask;

#[cw_serde]
pub struct ManagerRemoveTask {
    pub task_hash: Vec<u8>,
}

impl ManagerRemoveTask {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = RemoveTaskMsg::RemoveTask(self);
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

// This is just a helper to properly serialize the above message
#[cw_serde]
enum RemoveTaskMsg {
    RemoveTask(ManagerRemoveTask),
}

#[cw_serde]
pub struct ManagerCreateTaskBalance {
    pub task_hash: Vec<u8>,
    pub amount_for_one_task: AmountForOneTask,
}

impl ManagerCreateTaskBalance {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = CreateTaskBalanceMsg::CreateTaskBalance(self);
        to_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>>(self, contract_addr: T, funds: Vec<Coin>) -> StdResult<CosmosMsg> {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg,
            funds,
        };
        Ok(execute.into())
    }
}

#[cw_serde]
enum CreateTaskBalanceMsg {
    CreateTaskBalance(ManagerCreateTaskBalance),
}
