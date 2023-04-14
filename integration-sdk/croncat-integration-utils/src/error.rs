use cosmwasm_std::{Addr, StdError, Uint64};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CronCatContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Reply {reply_id} failed")]
    ReplyError { reply_id: Uint64 },

    #[error("Failed to retrieve latest task execution info from potential CronCat manager: {manager_addr}")]
    LatestTaskInfoFailed { manager_addr: Addr },

    #[error("Could not deserialize task info")]
    DeserializeTaskInfo {},

    #[error("No response from factory regarding manager version")]
    FactoryManagerQueryFailed { manager_addr: Addr, version: String },

    #[error("Attempted invocation from unsanctioned manager contract")]
    UnsanctionedInvocation { manager_addr: Addr, version: String },

    #[error("Invocation not in the same block and transaction index")]
    NotSameBlockTxIndex {},

    #[error("Invocation not called by task owner. Expected owner: {expected_owner}")]
    WrongTaskOwner { expected_owner: Addr },
}
