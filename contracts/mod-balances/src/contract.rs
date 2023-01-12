use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
#[cfg(not(feature = "library"))]
use cw2::set_contract_version;
use cw20::{Balance, BalanceResponse};
use mod_sdk::types::QueryResponse;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::types::{BalanceComparator, HasBalanceComparator};
use crate::ContractError;

// version info for migration info
const CONTRACT_NAME: &str = "croncat:mod-balances";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, StdError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Err(ContractError::Noop)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBalance { address, denom } => {
            to_binary(&query_get_balance(deps, address, denom)?)
        }
        QueryMsg::GetCw20Balance {
            cw20_contract,
            address,
        } => to_binary(&query_get_cw20_balance(deps, cw20_contract, address)?),
        QueryMsg::HasBalanceComparator(HasBalanceComparator {
            address,
            required_balance,
            comparator,
        }) => to_binary(&query_has_balance_comparator(
            deps,
            address,
            required_balance,
            comparator,
        )?),
    }
}

/// Query: GetBalance
/// Used as a helper method to get the native balance for an account
///
/// Response: QueryResponse
/// Always returns true, even if balance is 0
/// Data is the balance found for this account
fn query_get_balance(deps: Deps, address: String, denom: String) -> StdResult<QueryResponse> {
    let valid_addr = deps.api.addr_validate(&address)?;
    let coin = deps.querier.query_balance(valid_addr, denom)?;
    Ok(QueryResponse {
        result: true,
        data: to_binary(&coin)?,
    })
}

/// Query: GetCw20Balance
/// Used as a helper method to get the CW20 balance for an account
///
/// Response: QueryResponse
/// Always returns true, even if balance is 0
/// Data is the balance found for this account
fn query_get_cw20_balance(
    deps: Deps,
    cw20_contract: String,
    address: String,
) -> StdResult<QueryResponse> {
    let valid_cw20 = deps.api.addr_validate(&cw20_contract)?;
    let valid_address = deps.api.addr_validate(&address)?;
    let balance_response: BalanceResponse = deps.querier.query_wasm_smart(
        valid_cw20,
        &cw20::Cw20QueryMsg::Balance {
            address: valid_address.to_string(),
        },
    )?;
    let coin = coin(balance_response.balance.into(), cw20_contract);
    Ok(QueryResponse {
        result: true,
        data: to_binary(&coin)?,
    })
}

/// Query: HasBalanceComparator
/// Used for comparing on-chain balance with a pre-defined input balance
/// Comparator allows the flexibility of a single method implementation
/// for all types of comparators: Equal, Not Equal, Greater Than,
/// Greater Than Equal To, Less Than, Less Than Equal To
/// If address doesn't exist, the query works as if the balance is zero
///
/// Response: QueryResponse
/// Will never error, but default to returning false for logical use.
fn query_has_balance_comparator(
    deps: Deps,
    address: String,
    required_balance: Balance,
    comparator: BalanceComparator,
) -> StdResult<QueryResponse> {
    let valid_address = deps.api.addr_validate(&address)?;

    // NOTE: This implementation requires only 1 coin to be compared.
    let (balance_amount, required_amount) = match required_balance {
        Balance::Native(required_native) => {
            // Get the required denom from required_balance
            // then loop the queried chain balances to find matching required denom
            let native_vec = required_native.into_vec();
            let native = native_vec.first().cloned();
            if let Some(native) = native {
                let balance = deps.querier.query_balance(valid_address, native.denom)?;
                (balance.amount, native.amount)
            } else {
                return Ok(QueryResponse {
                    result: false,
                    data: Default::default(),
                });
            }
        }
        Balance::Cw20(required_cw20) => {
            let balance_response: BalanceResponse = deps.querier.query_wasm_smart(
                required_cw20.address.clone(),
                &cw20::Cw20QueryMsg::Balance { address },
            )?;
            (balance_response.balance, required_cw20.amount)
        }
    };

    let result = match comparator {
        BalanceComparator::Eq => required_amount == balance_amount,
        BalanceComparator::Ne => required_amount != balance_amount,
        BalanceComparator::Gt => required_amount < balance_amount,
        BalanceComparator::Gte => required_amount <= balance_amount,
        BalanceComparator::Lt => required_amount > balance_amount,
        BalanceComparator::Lte => required_amount >= balance_amount,
    };

    Ok(QueryResponse {
        result,
        data: to_binary(&balance_amount)?,
    })
}
