// use schemars::JsonSchema;
// use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, has_coins, to_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult,
};
use cw2::set_contract_version;
use cw20::BalanceResponse;
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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBalance { address, denom } => {
            to_binary(&query_get_balance(deps, address, denom)?)
        }
        QueryMsg::GetCW20Balance {
            cw20_contract,
            address,
        } => to_binary(&query_get_cw20_balance(deps, cw20_contract, address)?),
        QueryMsg::HasBalance {
            balance,
            required_balance,
        } => to_binary(&query_has_balance(balance, required_balance)?),
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
    cw20_contract: Addr,
    address: Addr,
) -> StdResult<RuleResponse<Option<Binary>>> {
    let balance: BalanceResponse = deps.querier.query_wasm_smart(
        cw20_contract,
        &Balance {
            address: address.to_string(),
        },
    )?;
    let amount = to_binary(&balance.balance).ok();
    Ok((true, amount))
}

fn query_has_balance(
    balance: Vec<Coin>,
    required_balance: Coin,
) -> StdResult<RuleResponse<Option<Binary>>> {
    Ok((has_coins(&balance, &required_balance), None))
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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{coin, coins, to_binary, Addr, Empty};
    use cw20::Cw20Coin;
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    pub fn cw20_template() -> Box<dyn Contract<Empty>> {
        let cw20 = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        );
        Box::new(cw20)
    }

    const ADMIN: &str = "cosmos1sjllsnramtg3ewxqwwrwjxfgc4n4ef9u0tvx7u";
    const ANYONE: &str = "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
    const ADMIN_CW20: &str = "cosmos1a7uhnpqthunr2rzj0ww0hwurpn42wyun6c5puz";
    const ADMIN_CW721: &str = "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
    const NATIVE_DENOM: &str = "atom";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            let accounts: Vec<(u128, String)> = vec![
                (6_000_000, ADMIN.to_string()),
                (6_000_000, ADMIN_CW20.to_string()),
                (6_000_000, ADMIN_CW721.to_string()),
                (1_000_000, ANYONE.to_string()),
            ];
            for (amt, address) in accounts.iter() {
                router
                    .bank
                    .init_balance(
                        storage,
                        &Addr::unchecked(address),
                        vec![coin(amt.clone(), NATIVE_DENOM.to_string())],
                    )
                    .unwrap();
            }
        })
    }

    fn proper_instantiate() -> (App, Addr, Addr) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());
        let owner_addr = Addr::unchecked(ADMIN);
        let nft_owner_addr = Addr::unchecked(ADMIN_CW20);

        let msg = InstantiateMsg {};
        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                owner_addr,
                &msg,
                &coins(2_000_000, NATIVE_DENOM),
                "CW-RULES",
                None,
            )
            .unwrap();

        let cw20_id = app.store_code(cw20_template());
        let msg = cw20_base::msg::InstantiateMsg {
            name: "Test".to_string(),
            symbol: "Test".to_string(),
            decimals: 6,
            initial_balances: vec![Cw20Coin {
                address: ANYONE.to_string(),
                amount: 15u128.into(),
            }],
            mint: None,
            marketing: None,
        };
        let cw20_addr = app
            .instantiate_contract(cw20_id, nft_owner_addr, &msg, &[], "Fungible-tokens", None)
            .unwrap();

        (app, cw_template_contract_addr, cw20_addr)
    }

    #[test]
    fn test_query_get_balance() -> StdResult<()> {
        let (app, contract_addr, _) = proper_instantiate();

        let msg = QueryMsg::GetBalance {
            address: Addr::unchecked(ANYONE),
            denom: NATIVE_DENOM.to_string(),
        };

        let res: RuleResponse<Option<Binary>> = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &msg)
            .unwrap();

        assert!(res.0);
        assert_eq!(res.1.unwrap(), to_binary("1000000")?);

        let msg = QueryMsg::GetBalance {
            address: Addr::unchecked(ANYONE),
            denom: "juno".to_string(),
        };
        let res: RuleResponse<Option<Binary>> =
            app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();

        assert!(res.0);
        assert_eq!(res.1.unwrap(), to_binary("0")?);

        Ok(())
    }

    #[test]
    fn query_get_cw20_balance() -> StdResult<()> {
        let (app, contract_addr, cw20_contract) = proper_instantiate();

        let msg = QueryMsg::GetCW20Balance {
            cw20_contract,
            address: Addr::unchecked(ANYONE),
        };

        let res: RuleResponse<Option<Binary>> =
            app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();

        assert!(res.0);
        assert_eq!(res.1.unwrap(), to_binary("15")?);

        Ok(())
    }
}
