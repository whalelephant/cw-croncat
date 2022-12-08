use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CoreError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Boundary is not in valid format")]
    InvalidBoundary {},

    #[error("No coin balance found")]
    EmptyBalance {},

    #[error("Not enough cw20 balance of {addr}, need {lack} more")]
    NotEnoughCw20 { addr: String, lack: Uint128 },

    #[error("Not enough native balance of {denom}, need {lack} more")]
    NotEnoughNative { denom: String, lack: Uint128 },

    #[error("invalid cosmwasm message")]
    InvalidWasmMsg {},

    #[error("Actions message unsupported or invalid message data")]
    InvalidAction {},

    #[error("Invalid gas input")]
    InvalidGas {},

    #[error("Task({task_hash}) became invalid after replacing placeholder")]
    TaskNoLongerValid { task_hash: String },
    #[error("Must provide gas limit for WASM actions")]
    NoGasLimit {},
}
