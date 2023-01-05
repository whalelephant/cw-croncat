use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("No coin balance found")]
    EmptyBalance {},

    #[error("Invalid gas_price")]
    InvalidGasPrice {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Contract paused")]
    Paused {},

    #[error("Must not attach funds")]
    RedundantFunds {},
}
