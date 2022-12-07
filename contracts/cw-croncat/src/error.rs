use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    CoreError(#[from] cw_croncat_core::error::CoreError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("An unknown reply ID was received.")]
    UnknownReplyID {},

    #[error("No task found by hash")]
    NoTaskFound {},

    #[error("Only accepts tokens in the cw20_whitelist")]
    NotInWhitelist {},

    #[error("Agent is not in the list of active agents")]
    AgentNotActive {},

    #[error("Agent not registered")]
    AgentNotRegistered {},

    #[error("{val:?} is paused")]
    ContractPaused { val: String },

    #[error("Can't attach deposit")]
    AttachedDeposit {},

    #[error("Only owner can refill their task")]
    RefillNotTaskOwner {},

    #[error("Rules are not ready. Failed at rule {index:?}")]
    RulesNotReady { index: u64 },

    #[error("No rules for this task hash: {task_hash}")]
    NoRulesForThisTask { task_hash: String },

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
