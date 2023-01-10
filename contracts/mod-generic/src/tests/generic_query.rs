use cosmwasm_std::{to_binary, Binary, WasmQuery};
use mod_sdk::types::QueryResponse;
use serde_json::json;

use crate::msg::QueryMsg;

use crate::tests::helpers::proper_instantiate;
use crate::types::{GenericQuery, ValueIndex};
use crate::value_ordering::ValueOrdering;

#[test]
fn test_generic() {
    let (app, contract_addr, cw4_addr, _) = proper_instantiate();

    let generic_query = GenericQuery {
        msg: to_binary(&cw4::Cw4QueryMsg::ListMembers {
            start_after: None,
            limit: None,
        })
        .unwrap(),
        path_to_value: vec![
            ValueIndex::Key("members".to_string()),
            ValueIndex::Index(1),
            ValueIndex::Key("weight".to_string()),
        ]
        .into(),
        ordering: ValueOrdering::UnitAbove,
        value: to_binary(&1).unwrap(),
        contract_addr: cw4_addr.into_string(),
    };
    let msg = QueryMsg::GenericQuery(generic_query);
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(res.result);
}

#[test]
fn test_generic_json_repr() {
    let (app, contract_addr, cw4_addr, _) = proper_instantiate();

    let query_binary = to_binary(&cw4::Cw4QueryMsg::ListMembers {
        start_after: None,
        limit: None,
    })
    .unwrap();
    let generic_query_json = json!({
        "generic_query": {
            "contract_addr": cw4_addr.to_string(),
            "msg": query_binary,
            "path_to_value": [{"key": "members"}, {"index": 1}, {"key": "weight"}],
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
    let res: QueryResponse = app.wrap().query(&request).unwrap();
    assert!(res.result);
}

#[test]
fn test_generic_bigint() {
    let (app, contract_addr, _, cw20_addr) = proper_instantiate();

    let generic_query = GenericQuery {
        msg: to_binary(&cw20::Cw20QueryMsg::TokenInfo {}).unwrap(),
        path_to_value: vec![ValueIndex::Key("total_supply".to_string())].into(),
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
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(res.result);
}
