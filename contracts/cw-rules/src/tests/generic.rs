use cosmwasm_std::{to_binary, Addr, Binary, Empty, WasmQuery};
use cw20::Cw20Coin;
use cw4::Member;
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use serde_json::json;

use crate::msg::{InstantiateMsg, QueryMsg, RuleResponse};
use generic_query::{GenericQuery, ValueIndex, ValueOrdering};

const CREATOR_ADDR: &str = "creator";

fn cw_rules_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

fn cw4_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw4_group::contract::execute,
        cw4_group::contract::instantiate,
        cw4_group::contract::query,
    );
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

#[test]
fn test_generic() {
    let mut app = App::default();
    let code_id = app.store_code(cw_rules_contract());
    let cw4_id = app.store_code(cw4_contract());

    let instantiate = InstantiateMsg {};
    let contract_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(CREATOR_ADDR),
            &instantiate,
            &[],
            "cw-rules",
            None,
        )
        .unwrap();

    let instantiate_cw4 = cw4_group::msg::InstantiateMsg {
        admin: None,
        // This is exactly what we get in the response, before 'get' chain
        // so to find bob's weight we need to get:
        // 1) key "members"
        // 2) 1 element of array(counting from zero)
        // 3) key "weight"
        members: vec![
            Member {
                addr: "alice".to_string(),
                weight: 1,
            },
            Member {
                addr: "bob".to_string(),
                weight: 2,
            },
        ],
    };
    let cw4_addr = app
        .instantiate_contract(
            cw4_id,
            Addr::unchecked(CREATOR_ADDR),
            &instantiate_cw4,
            &[],
            "cw4-group",
            None,
        )
        .unwrap();

    let generic_query = GenericQuery {
        msg: to_binary(&cw4::Cw4QueryMsg::ListMembers {
            start_after: None,
            limit: None,
        })
        .unwrap(),
        gets: vec![
            ValueIndex::Key("members".to_string()),
            ValueIndex::Index(1),
            ValueIndex::Key("weight".to_string()),
        ],
        ordering: ValueOrdering::UnitAbove,
        value: to_binary(&1).unwrap(),
        contract_addr: cw4_addr.into_string(),
    };
    let msg = QueryMsg::GenericQuery(generic_query);
    let ser = serde_json::to_string_pretty(&msg).unwrap();
    println!("{ser}");
    let res: RuleResponse<Option<Binary>> =
        app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(res.0);
}

#[test]
fn test_generic_json_repr() {
    let mut app = App::default();
    let code_id = app.store_code(cw_rules_contract());
    let cw4_id = app.store_code(cw4_contract());

    let instantiate = InstantiateMsg {};
    let contract_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(CREATOR_ADDR),
            &instantiate,
            &[],
            "cw-rules",
            None,
        )
        .unwrap();

    let instantiate_cw4 = cw4_group::msg::InstantiateMsg {
        admin: None,
        members: vec![
            Member {
                addr: "alice".to_string(),
                weight: 1,
            },
            Member {
                addr: "bob".to_string(),
                weight: 2,
            },
        ],
    };
    let cw4_addr = app
        .instantiate_contract(
            cw4_id,
            Addr::unchecked(CREATOR_ADDR),
            &instantiate_cw4,
            &[],
            "cw4-group",
            None,
        )
        .unwrap();

    let query_binary = to_binary(&cw4::Cw4QueryMsg::ListMembers {
        start_after: None,
        limit: None,
    })
    .unwrap();
    let generic_query_json = json!({
        "generic_query": {
            "contract_addr": cw4_addr.to_string(),
            "msg": query_binary,
            "gets": [{"key": "members"}, {"index": 1}, {"key": "weight"}],
            "ordering": "unit_above",
            "value": to_binary(&1).unwrap(),
        }
    });
    let msg = generic_query_json.to_string().into_bytes();
    let request = WasmQuery::Smart {
        contract_addr: contract_addr.into(),
        msg: Binary(msg),
    }
    .into();
    let res: RuleResponse<Option<Binary>> = app.wrap().query(&request).unwrap();
    assert!(res.0);
}

#[test]
fn test_generic_bigint() {
    let mut app = App::default();
    let code_id = app.store_code(cw_rules_contract());
    let cw20_id = app.store_code(cw20_template());

    let instantiate = InstantiateMsg {};
    let contract_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(CREATOR_ADDR),
            &instantiate,
            &[],
            "cw-rules",
            None,
        )
        .unwrap();

    let instantiate_cw20 = cw20_base::msg::InstantiateMsg {
        name: "test".to_string(),
        symbol: "hello".to_string(),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: CREATOR_ADDR.to_string(),
            amount: 2022_u128.into(),
        }],
        mint: None,
        marketing: None,
    };
    let cw20_addr = app
        .instantiate_contract(
            cw20_id,
            Addr::unchecked(CREATOR_ADDR),
            &instantiate_cw20,
            &[],
            "cw20-base",
            None,
        )
        .unwrap();

    let generic_query = GenericQuery {
        msg: to_binary(&cw20::Cw20QueryMsg::TokenInfo {}).unwrap(),
        gets: vec![ValueIndex::Key("total_supply".to_string())],
        ordering: ValueOrdering::UnitAbove,
        value: to_binary("2012").unwrap(),
        contract_addr: cw20_addr.into_string(),
    };
    // what we get here is :
    // pub struct TokenInfoResponse {
    //     pub name: String,
    //     pub symbol: String,
    //     pub decimals: u8,
    //     pub total_supply: Uint128,
    // }
    let msg = QueryMsg::GenericQuery(generic_query);
    let res: RuleResponse<Option<Binary>> =
        app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(res.0);
}
