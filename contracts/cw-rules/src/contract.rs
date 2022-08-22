// use schemars::JsonSchema;
// use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, to_vec, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo,
    QuerierWrapper, QueryRequest, Response, StdError, StdResult, WasmQuery,
};
use cw2::set_contract_version;
use cw_croncat_core::types::Rule;

use crate::error::ContractError;
use crate::helpers::{ValueOrd, ValueOrdering};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, QueryMultiResponse, RuleResponse};
use crate::types::{GenericQuery, ValueIndex};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-rules";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // Echo
        ExecuteMsg::QueryResult {} => query_result(deps, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBalance { address } => to_binary(&query_get_balance(env, address)?),
        QueryMsg::GetCW20Balance { address } => to_binary(&query_get_cw20_balance(env, address)?),
        QueryMsg::CheckOwnerOfNFT {
            address,
            nft_address,
            token_id,
        } => to_binary(&query_check_owner_nft(address, nft_address, token_id)?),
        QueryMsg::CheckProposalReadyToExec {
            dao_address,
            proposal_id,
        } => to_binary(&query_dao_proposal_ready(dao_address, proposal_id)?),
        QueryMsg::QueryConstruct { rules } => to_binary(&query_construct(deps, rules)?),
    }
}

pub fn query_result(_deps: DepsMut, _info: MessageInfo) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "query_result"))
}

// TODO:
fn query_get_balance(env: Env, address: Addr) -> StdResult<RuleResponse<Option<Binary>>> {
    Ok((true, None))
}

// TODO:
fn query_get_cw20_balance(env: Env, address: Addr) -> StdResult<RuleResponse<Option<Binary>>> {
    Ok((true, None))
}

// TODO:
fn query_check_owner_nft(
    address: Addr,
    nft_address: Addr,
    token_id: String,
) -> StdResult<RuleResponse<Option<Binary>>> {
    // let res: RuleResponse<Option<Binary>> = deps
    //     .querier
    //     .query_wasm_smart(nft_address, &msg)?;
    Ok((true, None))
}

// TODO:
fn query_dao_proposal_ready(
    dao_address: Addr,
    proposal_id: String,
) -> StdResult<RuleResponse<Option<Binary>>> {
    // let res: RuleResponse<Option<Binary>> = deps
    //     .querier
    //     .query_wasm_smart(dao_address, &msg)?;
    Ok((true, None))
}

// // GOAL:
// // Parse a generic query response, and inject input for the next query
// fn query_chain(deps: Deps, env: Env) -> StdResult<QueryMultiResponse> {
//     // Get known format for first msg
//     let msg1 = QueryMsg::GetRandom {};
//     let res1: RandomResponse = deps
//         .querier
//         .query_wasm_smart(&env.contract.address, &msg1)?;

//     // Query a bool with some data from previous
//     let msg2 = QueryMsg::GetBoolBinary {
//         msg: Some(to_binary(&res1)?),
//     };
//     let res2: RuleResponse<Option<Binary>> = deps
//         .querier
//         .query_wasm_smart(&env.contract.address, &msg2)?;

//     // Utilize previous query for the input of this query
//     // TODO: Setup binary msg, parse into something that contains { msg }, then assign the new binary response to it (if any)
//     // let msg = QueryMsg::GetInputBoolBinary {
//     //     msg: Some(to_binary(&res2)?),
//     // };
//     // let res: RuleResponse<Option<Binary>> =
//     //     deps.querier.query_wasm_smart(&env.contract.address, &msg)?;

//     // Format something to read results
//     let data = format!("{:?}", res2);
//     Ok(QueryMultiResponse { data })
// }

// create a smart query into binary
fn query_construct(deps: Deps, rules: Vec<Rule>) -> StdResult<bool> {
    for rule in rules {
        let contract_addr = deps.api.addr_validate(&rule.contract_addr)?.into_string();
        let query: GenericQuery = from_binary(&rule.msg)?;

        let request = QueryRequest::<Empty>::Wasm(WasmQuery::Smart {
            contract_addr,
            msg: to_binary(&cw4::Cw4QueryMsg::ListMembers {
                start_after: None,
                limit: None,
            })
            .unwrap(),
        });

        // Copied from `QuerierWrapper::query`
        // because serde_json_wasm fails to deserialize slice into `serde_json::Value`
        let raw = to_vec(&request).map_err(|serialize_err| {
            StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
        })?;
        let bin = match deps.querier.raw_query(&raw) {
            cosmwasm_std::SystemResult::Ok(cosmwasm_std::ContractResult::Ok(value)) => value,
            cosmwasm_std::SystemResult::Ok(cosmwasm_std::ContractResult::Err(contract_err)) => {
                return Err(StdError::generic_err(format!(
                    "Querier contract error: {}",
                    contract_err
                )));
            }
            cosmwasm_std::SystemResult::Err(system_err) => {
                return Err(StdError::generic_err(format!(
                    "Querier system error: {}",
                    system_err
                )));
            }
        };
        let json_val: Value = serde_json::from_slice(bin.as_slice())
            .map_err(|e| StdError::parse_err(std::any::type_name::<Value>(), e))?;
        println!("val: {json_val}");
        let mut current_val = &json_val;
        for get in query.gets {
            match get {
                ValueIndex::Key(s) => {
                    current_val = current_val
                        .get(s)
                        .ok_or_else(|| StdError::generic_err("Invalid key for value"))?
                }
                ValueIndex::Number(n) => {
                    current_val = current_val
                        .get(n as usize)
                        .ok_or_else(|| StdError::generic_err("Invalid index for value"))?
                }
            }
        }
        if !match query.ordering {
            ValueOrdering::UnitAbove => current_val.bt(&query.value)?,
            ValueOrdering::UnitAboveEqual => current_val.be(&query.value)?,
            ValueOrdering::UnitBelow => current_val.lt(&query.value)?,
            ValueOrdering::UnitBelowEqual => current_val.le(&query.value)?,
            ValueOrdering::Equal => current_val.eq(&query.value),
        } {
            return Ok(false);
        }
    }
    Ok(true)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
//     use cosmwasm_std::{coins, from_binary};

//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(1000, "earth"));

//         // we can just call .unwrap() to assert this was a success
//         let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // it worked, let's query the state
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(17, value.count);
//     }

//     #[test]
//     fn increment() {
//         let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Increment {};
//         let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // should increase counter by 1
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(18, value.count);
//     }
// }
