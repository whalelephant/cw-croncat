use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, Binary, CosmosMsg, StdResult, WasmMsg, Coin};
use cw20::Cw20CoinVerified;
use crate::types::AmountForOneTask;

#[cw_serde]
pub struct TaskCreatedHookMsg {}

impl TaskCreatedHookMsg {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = TaskCreatedHandleMsg::TaskCreatedHook(self);
        to_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg(self, contract_addr: String) -> StdResult<CosmosMsg> {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr,
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}
#[cw_serde]

enum TaskCreatedHandleMsg {
    TaskCreatedHook(TaskCreatedHookMsg),
}

#[cw_serde]
pub struct TaskCompletedHookMsg {
    pub is_block_slot_task: bool,
    pub agent_id: Addr,
}
impl TaskCompletedHookMsg {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = TaskCompletedHandleMsg::TaskCompletedHook(self);
        to_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg(self, contract_addr: String) -> StdResult<CosmosMsg> {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr,
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}
// This is just a helper to properly serialize the above message
#[cw_serde]
enum TaskCompletedHandleMsg {
    TaskCompletedHook(TaskCompletedHookMsg),
}

#[cw_serde]
pub struct WithdrawAgentRewardsHookMsg {
    pub agent_id: String,
    pub payable_account_id: String,
}
impl WithdrawAgentRewardsHookMsg {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = WithdrawAgentRewardsHandleMsg::WithdrawAgentRewardsHook(self);
        to_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg(self, contract_addr: String) -> StdResult<CosmosMsg> {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr,
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}

// This is just a helper to properly serialize the above message
#[cw_serde]
enum WithdrawAgentRewardsHandleMsg {
    WithdrawAgentRewardsHook(WithdrawAgentRewardsHookMsg),
}

//
#[cw_serde]
pub struct RemoveTaskHookMsg {
    pub task_hash: Vec<u8>,
    pub sender: Option<Addr>,
}

impl RemoveTaskHookMsg {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = RemoveTaskHandleMsg::RemoveTaskHook(self);
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
pub(crate) enum RemoveTaskHandleMsg {
    RemoveTaskHook(RemoveTaskHookMsg),
}

#[cw_serde]
pub struct RescheduleTaskHookMsg {
    pub task_hash: Vec<u8>,
}

impl RescheduleTaskHookMsg {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = RescheduleTaskHandleMsg::RescheduleTaskHook(self);
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
pub(crate) enum RescheduleTaskHandleMsg {
    RescheduleTaskHook(RescheduleTaskHookMsg),
}


// Note: sender and cw20 validated on the tasks contract
#[cw_serde]
pub struct CreateTaskBalanceHookMsg {
    pub sender: Addr,
    pub task_hash: Vec<u8>,
    pub recurring: bool,
    pub cw20: Option<Cw20CoinVerified>,
    pub amount_for_one_task: AmountForOneTask,
}

impl CreateTaskBalanceHookMsg {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = CreateTaskBalanceHandleMsg::CreateTaskBalanceHook(self);
        to_binary(&msg)
    }
    
    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>>(
        self,
        contract_addr: T,
        funds: Vec<Coin>,
    ) -> StdResult<CosmosMsg> {
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
enum CreateTaskBalanceHandleMsg {
    CreateTaskBalanceHook(CreateTaskBalanceHookMsg),
}
