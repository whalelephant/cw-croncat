use cosmwasm_std::{to_binary, Addr, Binary, Empty};
use cw20::Cw20Coin;
use cw4::Member;
use cw_croncat_core::types::Rule;
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use serde_json::json;

use crate::{
    helpers::ValueOrdering,
    msg::{InstantiateMsg, QueryMsg},
    types::generic_query::{GenericQuery, ValueIndex},
};

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
            ValueIndex::Number(1),
            ValueIndex::Key("weight".to_string()),
        ],
        ordering: ValueOrdering::UnitAbove,
        value: json!(1),
    };
    let binary = to_binary(&generic_query).unwrap();
    let msg = QueryMsg::QueryConstruct {
        rules: vec![Rule {
            contract_addr: cw4_addr.into_string(),
            msg: binary,
        }],
    };
    let res: bool = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(res);
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
        "msg": query_binary,
        "gets": ["members", 1, "weight"],
        "ordering": "unit_above",
        "value": 1
    });
    let binary: Binary = Binary(generic_query_json.to_string().into_bytes());
    let msg = QueryMsg::QueryConstruct {
        rules: vec![Rule {
            contract_addr: cw4_addr.into_string(),
            msg: binary,
        }],
    };
    let res: bool = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(res);
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
        value: json!("2012"),
    };
    // what we get here is :
    // pub struct TokenInfoResponse {
    //     pub name: String,
    //     pub symbol: String,
    //     pub decimals: u8,
    //     pub total_supply: Uint128,
    // }
    let binary = to_binary(&generic_query).unwrap();
    let msg = QueryMsg::QueryConstruct {
        rules: vec![Rule {
            contract_addr: cw20_addr.into_string(),
            msg: binary,
        }],
    };
    let res: bool = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(res);
}
