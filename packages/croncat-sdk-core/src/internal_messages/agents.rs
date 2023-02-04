use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, Binary, CosmosMsg, StdResult, WasmMsg};

#[cw_serde]
pub struct AgentOnTaskCreated {
    pub task_hash: String,
}

#[cw_serde]
pub struct AgentOnTaskCompleted {
    pub is_block_slot_task: bool,
    pub agent_id: Addr,
}

impl AgentOnTaskCreated {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = NewTaskMsg::OnTaskCreated(self);
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
enum NewTaskMsg {
    OnTaskCreated(AgentOnTaskCreated),
}
#[cw_serde]
pub struct AgentWithdrawOnRemovalArgs {
    pub agent_id: String,
    pub payable_account_id: String,
}
