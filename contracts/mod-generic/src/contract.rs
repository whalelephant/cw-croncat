#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
#[cfg(not(feature = "library"))]
use cw2::set_contract_version;
use mod_sdk::types::QueryResponse;

use crate::helpers::{bin_to_value, query_wasm_smart_raw};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::types::GenericQuery;
use crate::ContractError;

// version info for migration info
const CONTRACT_NAME: &str = "croncat:mod-generic";
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
        QueryMsg::GenericQuery(query) => to_binary(&generic_query(deps, query)?),
    }
}

/// Query: GenericQuery
/// Used for creating generic quieries
/// Parses the query result to receive the value according to the path, defined by `gets`
/// Compares this result with a pre-defined value
/// ValueOrdering allows several options for comparison:
/// Equal, Not Equal, Greater Than, Greater Than Equal To, Less Than, Less Than Equal To
///
/// Response: QueryResponse
/// Returns true if the pre-defined ordering is satisfied
/// Data contains the value which we received by querying
fn generic_query(deps: Deps, query: GenericQuery) -> StdResult<QueryResponse> {
    let mut json_val = query_wasm_smart_raw(deps, query.contract_addr, query.msg)
        .and_then(|bin| bin_to_value(bin.as_slice()))?;
    let json_rhs = cosmwasm_std::from_slice(query.value.as_slice())
        .map_err(|e| StdError::parse_err(std::any::type_name::<serde_cw_value::Value>(), e))?;
    let value = query.path_to_value.find_value(&mut json_val)?;

    let result = query.ordering.val_cmp(value, &json_rhs)?;
    Ok(QueryResponse {
        result,
        data: to_binary(&value)?,
    })
}
