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
}
