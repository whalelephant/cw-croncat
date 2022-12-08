use cosmwasm_std::{to_binary, Addr, Uint128};
use cw20::Cw20Coin;
use cw4::Member;
use cw_multi_test::{App, Executor};

use cw_rules_core::msg::{InstantiateMsg, QueryMsg, QueryResponse};
use generic_query::{PathToValue, ValueIndex, ValueOrdering};
use smart_query::{SmartQueries, SmartQuery, SmartQueryHead};

use crate::tests::helpers::{cw20_template, cw4_contract, cw_rules_contract, CREATOR_ADDR};

#[test]
fn test_smart() {
    let mut app = App::default();
    let code_id = app.store_code(cw_rules_contract());
    let cw4_id = app.store_code(cw4_contract());
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

    let instantiate_cw4 = cw4_group::msg::InstantiateMsg {
        admin: Some("aloha".to_owned()),
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

    let instantiate_cw20 = cw20_base::msg::InstantiateMsg {
        name: "test".to_string(),
        symbol: "hello".to_string(),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: "aloha".to_string(),
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

    let head_msg = cw4_group::msg::QueryMsg::Admin {};

    let queries = SmartQueries(vec![SmartQuery {
        contract_addr: cw20_addr.to_string(),
        msg: to_binary(&cw20_base::msg::QueryMsg::Balance {
            address: "lol".to_owned(),
        })
        .unwrap(),
        path_to_msg_value: PathToValue(vec![
            ValueIndex::from("balance".to_owned()),
            ValueIndex::from("address".to_owned()),
        ]),
        path_to_query_value: PathToValue(vec![ValueIndex::from("balance".to_owned())]),
    }]);
    let smart_query = SmartQueryHead {
        contract_addr: cw4_addr.to_string(),
        msg: to_binary(&head_msg).unwrap(),
        path_to_query_value: vec!["admin".to_owned().into()].into(),
        queries,
        ordering: ValueOrdering::Equal,
        value: to_binary(&Uint128::from(2022_u128)).unwrap(),
    };

    let msg = QueryMsg::SmartQuery(smart_query);
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert_eq!(
        res,
        QueryResponse {
            result: true,
            data: to_binary(&Uint128::from(2022_u128)).unwrap()
        }
    )
}
