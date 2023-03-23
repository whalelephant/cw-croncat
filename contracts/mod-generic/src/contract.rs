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
/// Returns true if the pre-defined ordering is satisfied
/// Data contains the LAST value which we received by querying
fn batch_query(deps: Deps, queries: Vec<CosmosQuery>) -> StdResult<Option<QueryResponse>> {
    // Process all the queries
    let mut last_response: Option<QueryResponse> = None;
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
                if q.check_result && !res.result {
                    last_response = None;
                    break;
                }
                last_response = Some(res);
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
                        // conform response to expected end result,
                        // always true since we just wanna use values in transforms later
                        match data {
                            Err(..) => {
                                last_response = None;
                            }
                            Ok(d) => {
                                last_response = Some(QueryResponse {
                                    result: true,
                                    data: to_binary(&d)?,
                                });
                            }
                        }
                    }
                    WasmQuery::Raw { contract_addr, key } => {
                        let res: Result<Option<Vec<u8>>, StdError> =
                            deps.querier.query_wasm_raw(contract_addr, key.clone());

                        match res {
                            Err(..) => {
                                last_response = None;
                            }
                            Ok(d) => {
                                // Optimistically respond
                                if let Some(r) = d {
                                    // conform response to expected end result,
                                    // always true since we just wanna use values in transforms later
                                    last_response = Some(QueryResponse {
                                        result: true,
                                        data: to_binary(&r)?,
                                    });
                                } else {
                                    last_response = Some(QueryResponse {
                                        result: true,
                                        data: Binary::default(),
                                    });
                                };
                            }
                        }
                    }
                    WasmQuery::ContractInfo { contract_addr } => {
                        let res = deps
                            .querier
                            .query_wasm_contract_info(contract_addr.clone().to_string())?;
                        // conform response to expected end result,
                        // always true since we just wanna use values in transforms later
                        last_response = Some(QueryResponse {
                            result: true,
                            data: to_binary(&res)?,
                        });
                    }
                    // NOTE: This is dependent on features = ["cosmwasm_1_2"]
                    WasmQuery::CodeInfo { code_id } => {
                        let res = deps.querier.query_wasm_code_info(*code_id)?;
                        // conform response to expected end result,
                        // always true since we just wanna use values in transforms later
                        last_response = Some(QueryResponse {
                            result: true,
                            data: to_binary(&res)?,
                        });
                    }
                    _ => {
                        println!("WasmQuery::Unknown");
                        return Err(StdError::GenericErr {
                            msg: "Unknown Query Type".to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(last_response)
}
