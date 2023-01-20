use cosmwasm_std::{to_binary, Binary, WasmQuery};
use mod_sdk::types::QueryResponse;
use serde_json::json;

use crate::msg::QueryMsg;

use crate::tests::helpers::proper_instantiate;
use crate::types::{GenericQuery, ValueIndex};
use crate::value_ordering::ValueOrdering;

#[test]
fn test_generic() {
    // Instantiate generic query contract and
    // cw4 contract with "alice" weight 1 and "bob" weight 2
    let (app, contract_addr, cw4_addr, _) = proper_instantiate();

    // We create a query that checks if the weight of the second member ("bob") is greater than 1
    // "msg" creates a query to "contract_addr" to list all members with their weights
    // The rusult is:
    // {
    //   "members": [
    //     {
    //       "addr": "alice",
    //       "weight": 1
    //     },
    //     {
    //       "addr": "bob",
    //       "weight": 2
    //     }
    //   ]
    // }
    // To get the weight of "bob" we specify the path to it in "path_to_value"
    // We compare it to "value"
    // Correlation "greater than" is defined by "ordering"

    // Tests with UnitAbove
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
        contract_addr: cw4_addr.to_string(),
    };
    let msg = QueryMsg::GenericQuery(generic_query);
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&2).unwrap());

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
        value: to_binary(&2).unwrap(),
        contract_addr: cw4_addr.to_string(),
    };
    let msg = QueryMsg::GenericQuery(generic_query);
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&2).unwrap());

    // Tests with UnitAboveEqual
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
        ordering: ValueOrdering::UnitAboveEqual,
        value: to_binary(&2).unwrap(),
        contract_addr: cw4_addr.to_string(),
    };
    let msg = QueryMsg::GenericQuery(generic_query);
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&2).unwrap());

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
        ordering: ValueOrdering::UnitAboveEqual,
        value: to_binary(&3).unwrap(),
        contract_addr: cw4_addr.into_string(),
    };
    let msg = QueryMsg::GenericQuery(generic_query);
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&2).unwrap());
}

#[test]
fn test_generic_json_repr() {
    // Instantiate generic query contract and
    // cw4 contract with "alice" weight 1 and "bob" weight 2
    let (app, contract_addr, cw4_addr, _) = proper_instantiate();

    // Tests with UnitBelow
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
            "ordering": "unit_below",
            "value": to_binary(&3).unwrap(),
        }
    });
    let msg = generic_query_json.to_string().into_bytes();
    let request = WasmQuery::Smart {
        contract_addr: contract_addr.to_string(),
        msg: Binary(msg),
    }
    .into();
    let res: QueryResponse = app.wrap().query(&request).unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&2).unwrap());

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
            "ordering": "unit_below",
            "value": to_binary(&2).unwrap(),
        }
    });
    let msg = generic_query_json.to_string().into_bytes();
    let request = WasmQuery::Smart {
        contract_addr: contract_addr.to_string(),
        msg: Binary(msg),
    }
    .into();
    let res: QueryResponse = app.wrap().query(&request).unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&2).unwrap());

    // Tests with UnitBelowEqual
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
            "ordering": "unit_below_equal",
            "value": to_binary(&2).unwrap(),
        }
    });
    let msg = generic_query_json.to_string().into_bytes();
    let request = WasmQuery::Smart {
        contract_addr: contract_addr.to_string(),
        msg: Binary(msg),
    }
    .into();
    let res: QueryResponse = app.wrap().query(&request).unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&2).unwrap());

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
            "ordering": "unit_below_equal",
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
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&2).unwrap());
}

#[test]
fn test_generic_bigint() {
    // Instantiate generic query and cw20 contracts with total supply equal to 2022
    let (app, contract_addr, _, cw20_addr) = proper_instantiate();

    // Tests with Equal
    let generic_query = GenericQuery {
        msg: to_binary(&cw20::Cw20QueryMsg::TokenInfo {}).unwrap(),
        path_to_value: vec![ValueIndex::Key("total_supply".to_string())].into(),
        ordering: ValueOrdering::Equal,
        value: to_binary("2022").unwrap(),
        contract_addr: cw20_addr.to_string(),
    };
    // what we get here is :
    // pub struct TokenInfoResponse {
    //     pub name: String,
    //     pub symbol: String,
    //     pub decimals: u8,
    //     pub total_supply: Uint128,
    // }
    let msg = QueryMsg::GenericQuery(generic_query);
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary("2022").unwrap());

    let generic_query = GenericQuery {
        msg: to_binary(&cw20::Cw20QueryMsg::TokenInfo {}).unwrap(),
        path_to_value: vec![ValueIndex::Key("total_supply".to_string())].into(),
        ordering: ValueOrdering::Equal,
        value: to_binary("2021").unwrap(),
        contract_addr: cw20_addr.to_string(),
    };
    let msg = QueryMsg::GenericQuery(generic_query);
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary("2022").unwrap());

    // Tests with NotEqual
    let generic_query = GenericQuery {
        msg: to_binary(&cw20::Cw20QueryMsg::TokenInfo {}).unwrap(),
        path_to_value: vec![ValueIndex::Key("total_supply".to_string())].into(),
        ordering: ValueOrdering::NotEqual,
        value: to_binary("2021").unwrap(),
        contract_addr: cw20_addr.to_string(),
    };
    let msg = QueryMsg::GenericQuery(generic_query);
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary("2022").unwrap());

    let generic_query = GenericQuery {
        msg: to_binary(&cw20::Cw20QueryMsg::TokenInfo {}).unwrap(),
        path_to_value: vec![ValueIndex::Key("total_supply".to_string())].into(),
        ordering: ValueOrdering::NotEqual,
        value: to_binary("2022").unwrap(),
        contract_addr: cw20_addr.into_string(),
    };
    let msg = QueryMsg::GenericQuery(generic_query);
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary("2022").unwrap());
}
