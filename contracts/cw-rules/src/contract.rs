// use schemars::JsonSchema;
// use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, BalanceResponse, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult,
};
use cw2::set_contract_version;
use cw20::Cw20QueryMsg::Balance;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, QueryMultiResponse, RuleResponse};

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
        QueryMsg::GetBalance { address, denom } => {
            to_binary(&query_get_balance(deps, env, address, denom)?)
        }
        QueryMsg::GetCW20Balance {
            cw20_contract,
            address,
        } => to_binary(&query_get_cw20_balance(deps, env, cw20_contract, address)?),
        QueryMsg::CheckOwnerOfNFT {
            address,
            nft_address,
            token_id,
        } => to_binary(&query_check_owner_nft(address, nft_address, token_id)?),
        QueryMsg::CheckProposalReadyToExec {
            dao_address,
            proposal_id,
        } => to_binary(&query_dao_proposal_ready(dao_address, proposal_id)?),
        // QueryMsg::QueryConstruct { rules } => to_binary(&query_construct(deps, env, rules)?),
    }
}

pub fn query_result(_deps: DepsMut, _info: MessageInfo) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "query_result"))
}

fn query_get_balance(
    deps: Deps,
    _env: Env,
    address: Addr,
    denom: String,
) -> StdResult<RuleResponse<Option<Binary>>> {
    let amount = deps
        .querier
        .query_balance(address, denom)
        .map(|coin| to_binary(&coin.amount))
        .unwrap()
        .ok();
    Ok((true, amount))
}

fn query_get_cw20_balance(
    deps: Deps,
    _env: Env,
    cw20_contract: Addr,
    address: Addr,
) -> StdResult<RuleResponse<Option<Binary>>> {
    let balance: BalanceResponse = deps.querier.query_wasm_smart(
        cw20_contract,
        &Balance {
            address: address.to_string(),
        },
    )?;
    let amount = to_binary(&balance.amount.amount).ok();
    Ok((true, amount))
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

// // // GOAL:
// // // Parse a generic query response, and inject input for the next query
// // fn query_chain(deps: Deps, env: Env) -> StdResult<QueryMultiResponse> {
// //     // Get known format for first msg
// //     let msg1 = QueryMsg::GetRandom {};
// //     let res1: RandomResponse = deps
// //         .querier
// //         .query_wasm_smart(&env.contract.address, &msg1)?;

// //     // Query a bool with some data from previous
// //     let msg2 = QueryMsg::GetBoolBinary {
// //         msg: Some(to_binary(&res1)?),
// //     };
// //     let res2: RuleResponse<Option<Binary>> = deps
// //         .querier
// //         .query_wasm_smart(&env.contract.address, &msg2)?;

// //     // Utilize previous query for the input of this query
// //     // TODO: Setup binary msg, parse into something that contains { msg }, then assign the new binary response to it (if any)
// //     // let msg = QueryMsg::GetInputBoolBinary {
// //     //     msg: Some(to_binary(&res2)?),
// //     // };
// //     // let res: RuleResponse<Option<Binary>> =
// //     //     deps.querier.query_wasm_smart(&env.contract.address, &msg)?;

// //     // Format something to read results
// //     let data = format!("{:?}", res2);
// //     Ok(QueryMultiResponse { data })
// // }

// // create a smart query into binary
// fn query_construct(_deps: Deps, _env: Env, _rules: Vec<Rule>) -> StdResult<QueryMultiResponse> {
//     let input_binary = to_binary(&RandomResponse { number: 1235 })?;

//     // create an injectable blank msg
//     let json_msg = json!({
//         "get_random": {
//             "msg": "",
//             "key": "value"
//         }
//     });
//     // blank msg to binary
//     let msg_binary = to_binary(&json_msg.to_string())?;

//     // try to parse binary
//     let msg_unbinary: String = from_binary(&msg_binary)?;
//     // let msg_parsed: Value = serde_json::from_str(msg_unbinary);
//     let msg_parse = serde_json::from_str(msg_unbinary.as_str());
//     let mut msg_parsed: String = "".to_string();

//     // Attempt to peel the onion, and fill with goodness
//     if let Ok(msg_parse) = msg_parse {
//         let parsed: Value = msg_parse;
//         // travel n1 child keys
//         if parsed.is_object() {
//             for (_key, value) in parsed.as_object().unwrap().iter() {
//                 for (k, _v) in value.as_object().unwrap().iter() {
//                     // check if this key has "msg"
//                     if k == "msg" {
//                         // then replace "msg" with "input_binary"
//                         // TODO:
//                         // parsed[key][k] = input_binary;
//                         msg_parsed = input_binary.to_string();
//                     }
//                 }
//             }
//         }
//     }

//     // Lastly, attempt to make the actual query!
//     // let res1 = deps
//     //     .querier
//     //     .query_wasm_smart(&env.contract.address, &msg1)?;

//     // Format something to read results
//     // let data = format!("{:?}", res1);
//     let data = format!("{:?} :: {:?}", msg_binary, msg_parsed);
//     Ok(QueryMultiResponse { data })
// }

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
