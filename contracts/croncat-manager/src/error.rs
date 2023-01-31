use cosmwasm_std::StdError;
use croncat_sdk_manager::SdkError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Sdk(#[from] SdkError),

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

    #[error("Only up to one ibc coin supported")]
    TooManyCoins {},

    #[error("Unknown task hash")]
    NoTaskHash {},

    #[error("Invalid version key, please update it")]
    InvalidKey {},

    // Note: this should never happen unless agent_fee + treasury_fee got compromised
    #[error("Invalid gas calculation")]
    InvalidGasCalculation {},

    #[error("No rewards available for withdraw")]
    NoWithdrawRewardsAvailable {},

    #[error("No rewards owner agent found")]
    NoRewardsOwnerAgentFound {},
}
