use cosmwasm_std::{Addr, StdError, Uint64};
use serde_json::Error as SerdeJsonError;
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

    #[error("Serialization error|{msg}")]
    SerdeError { msg: String },

    #[error("Must attach funds for task creation")]
    TaskCreationNoFunds,

    #[error("No contract named {contract_name} on factory {factory_addr}")]
    NoSuchContractOnFactory {
        contract_name: String,
        factory_addr: Addr,
    },
}

impl From<SerdeJsonError> for CronCatContractError {
    fn from(error: SerdeJsonError) -> Self {
        CronCatContractError::SerdeError {
            msg: format!("Serialization error: {}", error),
        }
    }
}
