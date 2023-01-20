use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Boundary is not in valid format")]
    InvalidBoundary {},

    #[error("Invalid interval")]
    InvalidInterval {},

    #[error("Empty balance, must attach funds")]
    MustAttach {},

    #[error("invalid cosmwasm message")]
    InvalidWasmMsg {},

    #[error("Actions message unsupported or invalid message data")]
    InvalidAction {},

    #[error("Invalid gas input")]
    InvalidGas {},

    #[error("Must provide gas limit for WASM actions")]
    NoGasLimit {},

    #[error("Contract is paused for actions")]
    Paused {},

    #[error("Task ended")]
    TaskEnded {},

    #[error("Task already exists")]
    TaskExists {},
}
