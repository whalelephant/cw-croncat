use cosmwasm_std::MessageInfo;
use croncat_sdk_core::types::Config;

use crate::ContractError;

/// Check if contract is paused or user attached redundant funds.
/// Called before every method, except [crate::contract::execute_update_config]
pub(crate) fn check_ready_for_execution(
    info: &MessageInfo,
    config: &Config,
) -> Result<(), ContractError> {
    if config.paused {
        Err(ContractError::Paused {})
    } else if !info.funds.is_empty() {
        Err(ContractError::RedundantFunds {})
    } else {
        Ok(())
    }
}
