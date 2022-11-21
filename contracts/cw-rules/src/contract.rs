use cw_rules_core::msg::{QueryConstruct, QueryConstructResponse};
use cw_rules_core::types::{CheckOwnerOfNft, CheckProposalStatus, HasBalanceGte, Queries};
// use schemars::JsonSchema;
// use serde::{Deserialize, Serialize};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, has_coins, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;
use cw20::{Balance, BalanceResponse, Cw20CoinVerified};
use cw721::Cw721QueryMsg::OwnerOf;
use cw721::OwnerOfResponse;
use smart_query::SmartQueryHead;

use crate::error::ContractError;
use crate::helpers::{bin_to_value, query_wasm_smart_raw};
use cw_rules_core::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, RuleResponse};

//use cosmwasm_std::from_binary;
//use crate::msg::QueryMultiResponse;
use crate::types::dao::{ProposalResponse, QueryDao, Status};
use generic_query::GenericQuery;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-rules";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBalance { address, denom } => {
            to_binary(&query_get_balance(deps, address, denom)?)
        }
        QueryMsg::GetCw20Balance {
            cw20_contract,
            address,
        } => to_binary(&query_get_cw20_balance(deps, cw20_contract, address)?),
        QueryMsg::HasBalanceGte(HasBalanceGte {
            address,
            required_balance,
        }) => to_binary(&query_has_balance_gte(deps, address, required_balance)?),
        QueryMsg::CheckOwnerOfNft(CheckOwnerOfNft {
            address,
            nft_address,
            token_id,
        }) => to_binary(&query_check_owner_nft(
            deps,
            address,
            nft_address,
            token_id,
        )?),
        QueryMsg::CheckProposalStatus(CheckProposalStatus {
            dao_address,
            proposal_id,
            status,
        }) => to_binary(&query_dao_proposal_status(
            deps,
            dao_address,
            proposal_id,
            status,
        )?),
        QueryMsg::GenericQuery(query) => to_binary(&generic_query(deps, query)?),
        QueryMsg::SmartQuery(query) => to_binary(&smart_query(deps, query)?),
        QueryMsg::QueryConstruct(QueryConstruct { rules }) => {
            to_binary(&query_construct(deps, rules)?)
        }
    }
}

pub fn query_result(_deps: DepsMut, _info: MessageInfo) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "query_result"))
}

fn query_get_balance(deps: Deps, address: String, denom: String) -> StdResult<RuleResponse> {
    let valid_addr = deps.api.addr_validate(&address)?;
    let coin = deps.querier.query_balance(valid_addr, denom)?;
    Ok(RuleResponse {
        result: true,
        data: to_binary(&coin)?,
    })
}

fn query_get_cw20_balance(
    deps: Deps,
    cw20_contract: String,
    address: String,
) -> StdResult<RuleResponse> {
    let valid_cw20 = deps.api.addr_validate(&cw20_contract)?;
    let valid_address = deps.api.addr_validate(&address)?;
    let balance_response: BalanceResponse = deps.querier.query_wasm_smart(
        valid_cw20,
        &cw20::Cw20QueryMsg::Balance {
            address: valid_address.to_string(),
        },
    )?;
    let coin = coin(balance_response.balance.into(), cw20_contract);
    Ok(RuleResponse {
        result: true,
        data: to_binary(&coin)?,
    })
}

fn query_has_balance_gte(
    deps: Deps,
    address: String,
    required_balance: Balance,
) -> StdResult<RuleResponse> {
    let valid_address = deps.api.addr_validate(&address)?;
    let balance;
    let res = match required_balance {
        Balance::Native(required_native) => {
            let balances = deps.querier.query_all_balances(valid_address)?;
            let required_vec = required_native.into_vec();
            let res = required_vec.iter().all(|required| {
                required.amount == Uint128::zero() || has_coins(&balances, required)
            });
            balance = Balance::from(balances);
            res
        }
        Balance::Cw20(required_cw20) => {
            let balance_response: BalanceResponse = deps.querier.query_wasm_smart(
                required_cw20.address.clone(),
                &cw20::Cw20QueryMsg::Balance { address },
            )?;
            balance = Balance::Cw20(Cw20CoinVerified {
                address: required_cw20.address,
                amount: balance_response.balance,
            });
            balance_response.balance >= required_cw20.amount
        }
    };
    Ok(RuleResponse {
        result: res,
        data: to_binary(&balance)?,
    })
}

fn query_check_owner_nft(
    deps: Deps,
    address: String,
    nft_address: String,
    token_id: String,
) -> StdResult<RuleResponse> {
    let valid_nft = deps.api.addr_validate(&nft_address)?;
    let res: OwnerOfResponse = deps.querier.query_wasm_smart(
        valid_nft,
        &OwnerOf {
            token_id,
            include_expired: None,
        },
    )?;
    Ok(RuleResponse {
        result: address == res.owner,
        data: to_binary(&res)?,
    })
}

fn query_dao_proposal_status(
    deps: Deps,
    dao_address: String,
    proposal_id: u64,
    status: Status,
) -> StdResult<RuleResponse> {
    let dao_addr = deps.api.addr_validate(&dao_address)?;
    let bin = query_wasm_smart_raw(
        deps,
        dao_addr,
        to_binary(&QueryDao::Proposal { proposal_id })?,
    )?;

    let resp: ProposalResponse = cosmwasm_std::from_binary(&bin)?;
    Ok(RuleResponse {
        result: resp.proposal.status == status,
        data: bin,
    })
}

// // // GOAL:
// // // Parse a generic query response, and inject input for the next query
// // fn query_chain(deps: Deps, env: Env) -> StdResult<QueryMultiResponse> {
// //     // Get known format for first msg
// //     let msg1 = QueryMsg::GetRandom {};
// //     let res1: RandomResponse = deps
// //         .querier
// //         .query_wasm_smart(&env.contract.address, &msg1)?;

//     // Query a bool with some data from previous
//     let msg2 = QueryMsg::GetBoolBinary {
//         msg: Some(to_binary(&res1)?),
//     };
//     let res2: RuleResponse = deps
//         .querier
//         .query_wasm_smart(&env.contract.address, &msg2)?;

//     // Utilize previous query for the input of this query
//     // TODO: Setup binary msg, parse into something that contains { msg }, then assign the new binary response to it (if any)
//     // let msg = QueryMsg::GetInputBoolBinary {
//     //     msg: Some(to_binary(&res2)?),
//     // };
//     // let res: RuleResponse =
//     //     deps.querier.query_wasm_smart(&env.contract.address, &msg)?;

//     // Format something to read results
//     let data = format!("{:?}", res2);
//     Ok(QueryMultiResponse { data })
// }

// create a smart query into binary
fn query_construct(deps: Deps, rules: Vec<Queries>) -> StdResult<QueryConstructResponse> {
    let mut data = Vec::with_capacity(rules.len());
    for (idx, rule) in rules.into_iter().enumerate() {
        let res = match rule {
            Queries::Query { contract_addr, msg } => Ok(RuleResponse {
                result: true,
                data: deps
                    .querier
                    .query_wasm_raw(contract_addr, msg)?
                    .map(Into::into)
                    // Why cosmwasm allows queries to return None?
                    .unwrap_or_default(),
            }),
            Queries::HasBalanceGte(HasBalanceGte {
                address,
                required_balance,
            }) => query_has_balance_gte(deps, address, required_balance),
            Queries::CheckOwnerOfNft(CheckOwnerOfNft {
                address,
                nft_address,
                token_id,
            }) => query_check_owner_nft(deps, address, nft_address, token_id),
            Queries::CheckProposalStatus(CheckProposalStatus {
                dao_address,
                proposal_id,
                status,
            }) => query_dao_proposal_status(deps, dao_address, proposal_id, status),
            Queries::GenericQuery(query) => generic_query(deps, query),
            Queries::SmartQuery(query) => smart_query(deps, query),
        }?;
        if !res.result {
            return Ok(QueryConstructResponse {
                result: res.result,
                data: vec![to_binary(&(idx as u64))?],
            });
        }
        data.push(res.data);
    }
    Ok(QueryConstructResponse { result: true, data })
}

fn smart_query(deps: Deps, query: SmartQueryHead) -> StdResult<RuleResponse> {
    let mut json_val = query_wasm_smart_raw(deps, query.contract_addr, query.msg)
        .and_then(|bin| bin_to_value(bin.as_slice()))?;
    let json_rhs = cosmwasm_std::from_binary(&query.value)
        .map_err(|e| StdError::parse_err(std::any::type_name::<serde_cw_value::Value>(), e))?;
    let mut head_val = query.path_to_query_value.find_value(&mut json_val)?;

    for mut smart in query.queries.0 {
        if let Some(path_to_msg_value) = smart.path_to_msg_value {
            let mut head_msg_val = cosmwasm_std::from_binary(&smart.msg).map_err(|e| {
                StdError::parse_err(std::any::type_name::<serde_cw_value::Value>(), e)
            })?;
            let msg_val = path_to_msg_value.find_value(&mut head_msg_val)?;
            *msg_val = head_val.clone();
            smart.msg = Binary(
                serde_json_wasm::to_vec(&head_msg_val)
                    .map_err(|e| StdError::generic_err(e.to_string()))?,
            );
        };
        json_val = query_wasm_smart_raw(deps, smart.contract_addr, smart.msg)
            .and_then(|bin| bin_to_value(bin.as_slice()))?;

        head_val = smart.path_to_query_value.find_value(&mut json_val)?;
    }

    let result = query.ordering.val_cmp(head_val, &json_rhs)?;
    Ok(RuleResponse {
        result,
        data: to_binary(&head_val)?,
    })
}

fn generic_query(deps: Deps, query: GenericQuery) -> StdResult<RuleResponse> {
    let mut json_val = query_wasm_smart_raw(deps, query.contract_addr, query.msg)
        .and_then(|bin| bin_to_value(bin.as_slice()))?;
    let json_rhs = cosmwasm_std::from_slice(query.value.as_slice())
        .map_err(|e| StdError::parse_err(std::any::type_name::<serde_cw_value::Value>(), e))?;
    let value = query.path_to_value.find_value(&mut json_val)?;

    let result = query.ordering.val_cmp(value, &json_rhs)?;
    Ok(RuleResponse {
        result,
        data: to_binary(&value)?,
    })
}
