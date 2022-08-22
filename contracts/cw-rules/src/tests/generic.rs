use cosmwasm_std::{to_binary, Addr, Binary, Empty};
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
