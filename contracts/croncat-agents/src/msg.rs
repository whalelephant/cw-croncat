use cosmwasm_schema::{cw_serde};

pub use croncat_sdk_agents::msg::{AgentExecuteMsg,AgentQueryMsg, AgentResponse, GetAgentIdsResponse};

#[cw_serde]
pub struct InstantiateMsg {}
