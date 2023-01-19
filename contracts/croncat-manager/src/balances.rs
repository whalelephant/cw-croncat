use cosmwasm_std::{
    from_binary, to_binary, Addr, BankMsg, Coin, Deps, DepsMut, MessageInfo, Order, Response,
    StdError, StdResult, Storage, Uint128, WasmMsg,
};
use croncat_sdk_core::types::{BalancesResponse, Config};
use cw20::{Cw20Coin, Cw20CoinVerified, Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::{
    helpers::check_ready_for_execution,
    msg::ReceiveMsg,
    state::{
        AVAILABLE_CW20_BALANCE, AVAILABLE_NATIVE_BALANCE, CONFIG, TEMP_BALANCES_CW20,
        TEMP_BALANCES_NATIVE,
    },
    ContractError,
};

// Helpers

pub(crate) fn add_available_native(storage: &mut dyn Storage, coin: &Coin) -> StdResult<Uint128> {
    let new_bal = AVAILABLE_NATIVE_BALANCE.update(storage, &coin.denom, |bal| {
        bal.unwrap_or_default()
            .checked_add(coin.amount)
            .map_err(StdError::overflow)
    })?;
    Ok(new_bal)
}

pub(crate) fn sub_available_native(
    storage: &mut dyn Storage,
    coin: &Coin,
) -> Result<Uint128, ContractError> {
    let current_balance = AVAILABLE_NATIVE_BALANCE.may_load(storage, &coin.denom)?;
    let new_bal = if let Some(balance) = current_balance {
        balance
            .checked_sub(coin.amount)
            .map_err(StdError::overflow)?
    } else {
        return Err(ContractError::EmptyBalance {});
    };

    if new_bal.is_zero() {
        AVAILABLE_NATIVE_BALANCE.remove(storage, &coin.denom);
    } else {
        AVAILABLE_NATIVE_BALANCE.save(storage, &coin.denom, &new_bal)?;
    }
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
) -> Result<Uint128, ContractError> {
    let current_balance = AVAILABLE_CW20_BALANCE.may_load(storage, &cw20.address)?;
    let new_bal = if let Some(balance) = current_balance {
        balance
            .checked_sub(cw20.amount)
            .map_err(StdError::overflow)?
    } else {
        return Err(ContractError::EmptyBalance {});
    };

    if new_bal.is_zero() {
        AVAILABLE_CW20_BALANCE.remove(storage, &cw20.address);
    } else {
        AVAILABLE_CW20_BALANCE.save(storage, &cw20.address, &new_bal)?;
    }
    Ok(new_bal)
}

// pub(crate) fn add_agent_native(
//     storage: &mut dyn Storage,
//     agent_addr: &Addr,
//     coin: &Coin,
// ) -> StdResult<Uint128> {
//     let new_bal = AGENT_BALANCES_NATIVE.update(
//         storage,
//         (agent_addr, &coin.denom),
//         |bal| -> StdResult<Uint128> {
//             let bal = bal.unwrap_or_default();
//             Ok(bal.checked_add(coin.amount)?)
//         },
//     )?;
//     Ok(new_bal)
// }

pub(crate) fn add_user_native(
    storage: &mut dyn Storage,
    user_addr: &Addr,
    coin: &Coin,
) -> StdResult<Uint128> {
    let new_bal = TEMP_BALANCES_NATIVE.update(
        storage,
        (user_addr, &coin.denom),
        |bal| -> StdResult<Uint128> {
            let bal = bal.unwrap_or_default();
            Ok(bal.checked_add(coin.amount)?)
        },
    )?;
    Ok(new_bal)
}

pub(crate) fn sub_user_native(
    storage: &mut dyn Storage,
    user_addr: &Addr,
    coin: &Coin,
) -> Result<Uint128, ContractError> {
    let current_balance = TEMP_BALANCES_NATIVE.may_load(storage, (user_addr, &coin.denom))?;
    let new_bal = if let Some(bal) = current_balance {
        bal.checked_sub(coin.amount).map_err(StdError::overflow)?
    } else {
        return Err(ContractError::EmptyBalance {});
    };

    if new_bal.is_zero() {
        TEMP_BALANCES_NATIVE.remove(storage, (user_addr, &coin.denom));
    } else {
        TEMP_BALANCES_NATIVE.save(storage, (user_addr, &coin.denom), &new_bal)?;
    }
    Ok(new_bal)
}

pub(crate) fn add_user_cw20(
    storage: &mut dyn Storage,
    user_addr: &Addr,
    cw20: &Cw20CoinVerified,
) -> StdResult<Uint128> {
    let new_bal = TEMP_BALANCES_CW20.update(
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
    let current_balance = TEMP_BALANCES_CW20.may_load(storage, (user_addr, &cw20.address))?;
    let new_bal = if let Some(bal) = current_balance {
        bal.checked_sub(cw20.amount).map_err(StdError::overflow)?
    } else {
        return Err(ContractError::EmptyBalance {});
    };

    if new_bal.is_zero() {
        TEMP_BALANCES_CW20.remove(storage, (user_addr, &cw20.address));
    } else {
        TEMP_BALANCES_CW20.save(storage, (user_addr, &cw20.address), &new_bal)?;
    }
    Ok(new_bal)
}

// Contract methods

/// Execute: Receive
/// Message validated, to be sure about intention of transferred tokens
/// Used by users before creating a task with cw20 send or transfer messages
///
/// Returns updated balances
pub fn execute_receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    let config = CONFIG.load(deps.storage)?;
    check_ready_for_execution(&info, &config)?;

    match msg {
        ReceiveMsg::RefillCw20Balance {} => {
            let sender = deps.api.addr_validate(&wrapper.sender)?;
            let coin_addr = info.sender;

            let cw20_verified = Cw20CoinVerified {
                address: coin_addr,
                amount: wrapper.amount,
            };
            let user_cw20_balance = add_user_cw20(deps.storage, &sender, &cw20_verified)?;
            let available_cw20_balance = add_available_cw20(deps.storage, &cw20_verified)?;
            Ok(Response::new()
                .add_attribute("action", "receive_cw20")
                .add_attribute("cw20_received", cw20_verified.to_string())
                .add_attribute("available_cw20_balance", available_cw20_balance)
                .add_attribute("user_cw20_balance", user_cw20_balance))
        }
    }
}

/// Execute: WithdrawCw20WalletBalances
/// Used by users to withdraw back their cw20 tokens
///
/// Returns updated balances
pub fn execute_user_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    native_balances: Vec<Coin>,
    cw20_balances: Vec<Cw20Coin>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    check_ready_for_execution(&info, &config)?;

    let user_addr = info.sender;
    let withdraws: Vec<Cw20CoinVerified> = cw20_balances
        .into_iter()
        .map(|cw20| {
            let address = deps.api.addr_validate(&cw20.address)?;
            Ok(Cw20CoinVerified {
                address,
                amount: cw20.amount,
            })
        })
        .collect::<StdResult<_>>()?;

    // update user and croncat manager balances
    for cw20 in withdraws.iter() {
        sub_user_cw20(deps.storage, &user_addr, cw20)?;
        sub_available_cw20(deps.storage, cw20)?;
    }

    for coin in native_balances.iter() {
        sub_user_native(deps.storage, &user_addr, coin)?;
        sub_available_native(deps.storage, coin)?;
    }

    let msgs = {
        let mut msgs = Vec::with_capacity(withdraws.len());
        for wd in withdraws {
            msgs.push(WasmMsg::Execute {
                contract_addr: wd.address.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: user_addr.to_string(),
                    amount: wd.amount,
                })?,
                funds: vec![],
            });
        }
        msgs
    };

    let response = Response::new()
        .add_attribute("action", "user_withdraw")
        .add_messages(msgs);

    if !native_balances.is_empty() {
        Ok(response.add_message(BankMsg::Send {
            to_address: user_addr.to_string(),
            amount: native_balances,
        }))
    } else {
        Ok(response)
    }
}

/// Execute: MoveBalances
/// Used by owner of the contract to move balances from the manager to treasury or owner address
pub fn execute_owner_withdraw(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner_addr {
        return Err(ContractError::Unauthorized {});
    }
    let address = config.treasury_addr.unwrap_or(config.owner_addr);
    let native_withdraw: Vec<Coin> = AVAILABLE_NATIVE_BALANCE
        .range(deps.storage, None, None, Order::Ascending)
        .map(|coin_res| coin_res.map(|(denom, amount)| Coin { denom, amount }))
        .collect::<StdResult<_>>()?;

    let cw20_withdraw: Vec<Cw20CoinVerified> = AVAILABLE_CW20_BALANCE
        .range(deps.storage, None, None, Order::Ascending)
        .map(|coin_res| coin_res.map(|(address, amount)| Cw20CoinVerified { address, amount }))
        .collect::<StdResult<_>>()?;

    AVAILABLE_NATIVE_BALANCE.clear(deps.storage);
    AVAILABLE_CW20_BALANCE.clear(deps.storage);

    let mut cw20_messages = Vec::with_capacity(cw20_withdraw.len());
    for cw20 in cw20_withdraw {
        cw20_messages.push(WasmMsg::Execute {
            contract_addr: cw20.address.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                recipient: address.to_string(),
                amount: cw20.amount,
            })?,
            funds: vec![],
        });
    }

    let response = Response::new()
        .add_attribute("action", "owner_withdraw")
        .add_messages(cw20_messages);
    if !native_withdraw.is_empty() {
        Ok(response.add_message(BankMsg::Send {
            to_address: address.to_string(),
            amount: native_withdraw,
        }))
    } else {
        Ok(response)
    }
}

/// Query: AvailableBalances
/// Used to get contract's available native and cw20 coins balances
/// Can be paginated
///
/// Returns list of native and cw20 balances
pub fn query_available_balances(
    deps: Deps,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> StdResult<BalancesResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    let from_index = from_index.unwrap_or_default();
    let limit = limit.unwrap_or(config.limit);

    let available_native_balance = AVAILABLE_NATIVE_BALANCE
        .range(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .map(|balance_res| balance_res.map(|(denom, amount)| Coin { denom, amount }))
        .collect::<StdResult<Vec<Coin>>>()?;

    let available_cw20_balance = AVAILABLE_CW20_BALANCE
        .range(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .map(|balance_res| {
            balance_res.map(|(address, amount)| Cw20CoinVerified { address, amount })
        })
        .collect::<StdResult<Vec<Cw20CoinVerified>>>()?;

    Ok(BalancesResponse {
        native_balance: available_native_balance,
        cw20_balance: available_cw20_balance,
    })
}

/// Query: Cw20WalletBalances
/// Used to get user's available cw20 coins balance that he can use to attach to the task balance
/// Can be paginated
///
/// Returns list of cw20 balances
pub fn query_users_balances(
    deps: Deps,
    wallet: String,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> StdResult<BalancesResponse> {
    let config = CONFIG.load(deps.storage)?;
    let addr = deps.api.addr_validate(&wallet)?;
    let from_index = from_index.unwrap_or_default();
    let limit = limit.unwrap_or(config.limit);

    let native_balance = TEMP_BALANCES_NATIVE
        .prefix(&addr)
        .range(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .map(|balance_res| balance_res.map(|(denom, amount)| Coin { denom, amount }))
        .collect::<StdResult<_>>()?;

    let cw20_balance = TEMP_BALANCES_CW20
        .prefix(&addr)
        .range(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .map(|balance_res| {
            balance_res.map(|(address, amount)| Cw20CoinVerified { address, amount })
        })
        .collect::<StdResult<_>>()?;

    Ok(BalancesResponse {
        native_balance,
        cw20_balance,
    })
}
