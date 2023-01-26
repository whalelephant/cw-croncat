use cosmwasm_std::{Coin, StdError};
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

    #[error("Agent is not active")]
    AgentNotActive,

    #[error("Insufficient funds. Needing {amount_needed:?}")]
    InsufficientFunds { amount_needed: Coin },

    #[error("Contract is in paused state")]
    ContractPaused,

    #[error("Not accepting new agents")]
    NotAcceptingNewAgents,

    #[error("Try again later for nomination")]
    TryLaterForNomination,

    #[error("Contract method does not accept any funds")]
    NoFundsShouldBeAttached,

    #[error("Unauthorized function call")]
    Unauthorized,

    #[error("No active agents in active agent list")]
    NoActiveAgents,

    #[error("Invalid CronCat manager address")]
    InvalidCroncatManagerAddress { addr: String },
}
