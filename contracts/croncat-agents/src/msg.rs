use cosmwasm_schema::{cw_serde, QueryResponses};

pub use croncat_sdk_agents::msg::{AgentExecuteMsg, AgentResponse, GetAgentIdsResponse};

#[cw_serde]
pub struct InstantiateMsg {}


#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
