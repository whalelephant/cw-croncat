#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, WasmQuery,
};
use cw2::set_contract_version;
use mod_sdk::types::QueryResponse;
use serde_cw_value::Value;

use crate::helpers::{bin_to_value, query_wasm_smart_raw};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::types::{CosmosQuery, GenericQuery};
use crate::ContractError;

// version info for migration info
const CONTRACT_NAME: &str = "crate:croncat-mod-generic";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, StdError> {
    let contract_version = msg.version.unwrap_or_else(|| CONTRACT_VERSION.to_string());
    set_contract_version(deps.storage, CONTRACT_NAME, &contract_version)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
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
        QueryMsg::BatchQuery { queries } => to_binary(&batch_query(deps, queries)?),
    }
}

/// Query: GenericQuery
/// Used for creating generic queries
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

/// Query an ordered set of cosmos queries
///
/// Response: QueryResponse
/// Returns true if the pre-defined ordering is satisfied across ALL queries
/// Data contains the array of values we received by querying
fn batch_query(deps: Deps, queries: Vec<CosmosQuery>) -> StdResult<QueryResponse> {
    // Optional here so we preserve request indexed responses
    let mut responses: Vec<Option<Binary>> = Vec::with_capacity(queries.len());
    let mut result = true;

    for query in &queries {
        match query {
            CosmosQuery::Croncat(q) => {
                let res: mod_sdk::types::QueryResponse = deps.querier.query(
                    &WasmQuery::Smart {
                        contract_addr: q.contract_addr.clone(),
                        msg: q.msg.clone(),
                    }
                    .into(),
                )?;
                // Collect all the dataz we canz
                responses.push(Some(res.data));

                // Only stop this train if a query result is false
                if q.check_result && !res.result {
                    result = res.result;
                    break;
                }
            }
            CosmosQuery::Wasm(wq) => {
                // Cover all native wasm query types
                match wq {
                    WasmQuery::Smart { contract_addr, msg } => {
                        let data: Result<Value, StdError> = deps.querier.query(
                            &WasmQuery::Smart {
                                contract_addr: contract_addr.clone().to_string(),
                                msg: msg.clone(),
                            }
                            .into(),
                        );
                        match data {
                            Err(..) => responses.push(None),
                            Ok(d) => {
                                responses.push(Some(to_binary(&d)?));
                            }
                        }
                    }
                    WasmQuery::Raw { contract_addr, key } => {
                        let res: Result<Option<Vec<u8>>, StdError> =
                            deps.querier.query_wasm_raw(contract_addr, key.clone());

                        match res {
                            Err(..) => responses.push(None),
                            Ok(d) => {
                                // Optimistically respond
                                if let Some(r) = d {
                                    responses.push(Some(to_binary(&r)?));
                                } else {
                                    responses.push(None);
                                };
                            }
                        }
                    }
                    WasmQuery::ContractInfo { contract_addr } => {
                        let res = deps
                            .querier
                            .query_wasm_contract_info(contract_addr.clone().to_string());
                        match res {
                            Err(..) => responses.push(None),
                            Ok(d) => {
                                responses.push(Some(to_binary(&d)?));
                            }
                        }
                    }
                    // // NOTE: This is dependent on features = ["cosmwasm_1_2"]
                    // WasmQuery::CodeInfo { code_id } => {
                    //     let res = deps.querier.query_wasm_code_info(*code_id);
                    //     match res {
                    //         Err(..) => responses.push(None),
                    //         Ok(d) => {
                    //             responses.push(Some(to_binary(&d)?));
                    //         }
                    //     }
                    // }
                    _ => {
                        return Err(StdError::GenericErr {
                            msg: "Unknown Query Type".to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(QueryResponse {
        result,
        data: to_binary(&responses)?,
    })
}
