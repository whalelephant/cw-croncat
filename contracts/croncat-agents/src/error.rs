use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    CoreError(#[from] croncat_sdk_agents::error::CoreError),

    #[error("Agent already registered")]
    AgentAlreadyRegistered,

    #[error("Agent not registered")]
    AgentNotRegistered,
}
