use cosmwasm_std::{Addr, Coin, Deps, MessageInfo, StdError, StdResult, Storage, Uint128};
use croncat_sdk_core::types::Config;
use cw20::Cw20CoinVerified;

use crate::{
    state::{
        AGENT_BALANCES_NATIVE, AVAILABLE_CW20_BALANCE, AVAILABLE_NATIVE_BALANCE,
        USERS_BALANCES_CW20,
    },
    ContractError,
};

pub(crate) fn add_available_native(storage: &mut dyn Storage, coin: &Coin) -> StdResult<Uint128> {
    let new_bal = AVAILABLE_NATIVE_BALANCE.update(storage, &coin.denom, |bal| {
        bal.unwrap_or_default()
            .checked_add(coin.amount)
            .map_err(StdError::overflow)
    })?;
    Ok(new_bal)
}

pub(crate) fn sub_available_native(storage: &mut dyn Storage, coin: &Coin) -> StdResult<Uint128> {
    let new_bal = AVAILABLE_NATIVE_BALANCE.update(storage, &coin.denom, |bal| {
        bal.unwrap_or_default()
            .checked_sub(coin.amount)
            .map_err(StdError::overflow)
    })?;
    Ok(new_bal)
}

pub(crate) fn add_available_cw20(
    storage: &mut dyn Storage,
    cw20: &Cw20CoinVerified,
) -> StdResult<Uint128> {
    let new_bal = AVAILABLE_CW20_BALANCE.update(storage, &cw20.address, |bal| {
        bal.unwrap_or_default()
            .checked_add(cw20.amount)
            .map_err(StdError::overflow)
    })?;
    Ok(new_bal)
}

pub(crate) fn sub_available_cw20(
    storage: &mut dyn Storage,
    cw20: &Cw20CoinVerified,
) -> StdResult<Uint128> {
    let new_bal = AVAILABLE_CW20_BALANCE.update(storage, &cw20.address, |bal| {
        bal.unwrap_or_default()
            .checked_sub(cw20.amount)
            .map_err(StdError::overflow)
    })?;
    Ok(new_bal)
}

pub(crate) fn add_agent_native(
    storage: &mut dyn Storage,
    agent_addr: &Addr,
    coin: &Coin,
) -> StdResult<Uint128> {
    let new_bal = AGENT_BALANCES_NATIVE.update(
        storage,
        (agent_addr, &coin.denom),
        |bal| -> StdResult<Uint128> {
            let bal = bal.unwrap_or_default();
            Ok(bal.checked_add(coin.amount)?)
        },
    )?;
    Ok(new_bal)
}

pub(crate) fn add_user_cw20(
    storage: &mut dyn Storage,
    user_addr: &Addr,
    cw20: &Cw20CoinVerified,
) -> StdResult<Uint128> {
    let new_bal = USERS_BALANCES_CW20.update(
        storage,
        (user_addr, &cw20.address),
        |bal| -> StdResult<Uint128> {
            let bal = bal.unwrap_or_default();
            Ok(bal.checked_add(cw20.amount)?)
        },
    )?;
    Ok(new_bal)
}

pub(crate) fn sub_user_cw20(
    storage: &mut dyn Storage,
    user_addr: &Addr,
    cw20: &Cw20CoinVerified,
) -> Result<Uint128, ContractError> {
    let current_balance = USERS_BALANCES_CW20.may_load(storage, (user_addr, &cw20.address))?;
    let mut new_bal = if let Some(bal) = current_balance {
        bal
    } else {
        return Err(ContractError::EmptyBalance {});
    };
    new_bal = new_bal
        .checked_sub(cw20.amount)
        .map_err(StdError::overflow)?;
    if new_bal.is_zero() {
        USERS_BALANCES_CW20.remove(storage, (user_addr, &cw20.address));
    } else {
        USERS_BALANCES_CW20.save(storage, (user_addr, &cw20.address), &new_bal)?;
    }
    Ok(new_bal)
}

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
