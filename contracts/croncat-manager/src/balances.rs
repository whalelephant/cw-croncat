use cosmwasm_std::{
    from_binary, to_binary, Addr, Coin, Deps, DepsMut, MessageInfo, Order, Response, StdError,
    StdResult, Storage, Uint128, WasmMsg,
};
use croncat_sdk_core::types::{BalancesResponse, Config};
use cw20::{Cw20Coin, Cw20CoinVerified, Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::{
    helpers::check_ready_for_execution,
    msg::ReceiveMsg,
    state::{AVAILABLE_CW20_BALANCE, AVAILABLE_NATIVE_BALANCE, CONFIG, USERS_BALANCES_CW20},
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

// pub(crate) fn sub_available_native(storage: &mut dyn Storage, coin: &Coin) -> StdResult<Uint128> {
//     let new_bal = AVAILABLE_NATIVE_BALANCE.update(storage, &coin.denom, |bal| {
//         bal.unwrap_or_default()
//             .checked_sub(coin.amount)
//             .map_err(StdError::overflow)
//     })?;
//     Ok(new_bal)
// }

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
    let new_bal = if let Some(bal) = current_balance {
        bal.checked_sub(cw20.amount).map_err(StdError::overflow)?
    } else {
        return Err(ContractError::EmptyBalance {});
    };

    if new_bal.is_zero() {
        USERS_BALANCES_CW20.remove(storage, (user_addr, &cw20.address));
    } else {
        USERS_BALANCES_CW20.save(storage, (user_addr, &cw20.address), &new_bal)?;
    }
    Ok(new_bal)
}

// Contract methods

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

pub fn execute_withdraw_wallet_balances(
    deps: DepsMut,
    info: MessageInfo,
    cw20_amounts: Vec<Cw20Coin>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    check_ready_for_execution(&info, &config)?;

    let wallet = info.sender;
    let withdraws: Vec<Cw20CoinVerified> = cw20_amounts
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
    let mut updated_user_cw20 = Vec::with_capacity(withdraws.len());
    let mut updated_available_cw20 = Vec::with_capacity(withdraws.len());
    for cw20 in withdraws.iter() {
        let new_user_bal = sub_user_cw20(deps.storage, &wallet, cw20)?;
        let new_avail_bal = sub_available_cw20(deps.storage, cw20)?;
        updated_user_cw20.push(Cw20CoinVerified {
            address: cw20.address.clone(),
            amount: new_user_bal,
        });
        updated_available_cw20.push(Cw20CoinVerified {
            address: cw20.address.clone(),
            amount: new_avail_bal,
        })
    }

    let msgs = {
        let mut msgs = Vec::with_capacity(withdraws.len());
        for wd in withdraws {
            msgs.push(WasmMsg::Execute {
                contract_addr: wd.address.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: wallet.to_string(),
                    amount: wd.amount,
                })?,
                funds: vec![],
            });
        }
        msgs
    };

    Ok(Response::new()
        .add_attribute("action", "withdraw_wallet_balances")
        .add_attribute(
            "updated_user_cw20_balances",
            format!("{updated_user_cw20:?}"),
        )
        .add_attribute(
            "updated_available_cw20_balances",
            format!("{updated_available_cw20:?}"),
        )
        .add_messages(msgs))
}

pub fn query_balances(
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
        native_denom: config.native_denom,
        available_native_balance,
        available_cw20_balance,
    })
}

pub fn query_cw20_wallet_balances(
    deps: Deps,
    wallet: String,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> StdResult<Vec<Cw20CoinVerified>> {
    let config = CONFIG.load(deps.storage)?;
    let addr = deps.api.addr_validate(&wallet)?;
    let from_index = from_index.unwrap_or_default();
    let limit = limit.unwrap_or(config.limit);

    let balances = USERS_BALANCES_CW20
        .prefix(&addr)
        .range(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .map(|balance_res| {
            balance_res.map(|(address, amount)| Cw20CoinVerified { address, amount })
        })
        .collect::<StdResult<_>>()?;

    Ok(balances)
}
