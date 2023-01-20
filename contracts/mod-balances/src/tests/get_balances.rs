use cosmwasm_std::{coin, to_binary, StdResult};
use mod_sdk::types::QueryResponse;

use crate::{
    msg::QueryMsg,
    tests::helpers::{proper_instantiate, ANOTHER},
};

use super::helpers::{ADMIN_CW20, ANYONE, NATIVE_DENOM};

#[test]
fn test_get_balance() -> StdResult<()> {
    let (app, contract_addr, _) = proper_instantiate();

    let msg = QueryMsg::GetBalance {
        address: ANYONE.to_string(),
        denom: NATIVE_DENOM.to_string(),
    };
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&coin(1000000, NATIVE_DENOM))?);

    // Balance with another denom is zero
    let msg = QueryMsg::GetBalance {
        address: ANYONE.to_string(),
        denom: "juno".to_string(),
    };
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&coin(0, "juno"))?);

    // Address doesn't exist, return zero balance
    let msg = QueryMsg::GetBalance {
        address: ANOTHER.to_string(),
        denom: NATIVE_DENOM.to_string(),
    };
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&coin(0, NATIVE_DENOM))?);

    Ok(())
}

#[test]
fn test_get_cw20_balance() -> StdResult<()> {
    let (app, contract_addr, cw20_contract) = proper_instantiate();

    // Return true and current balance
    let msg = QueryMsg::GetCw20Balance {
        cw20_contract: cw20_contract.to_string(),
        address: ANYONE.to_string(),
    };
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&coin(15, &cw20_contract))?);

    // Return coin if balance is zero
    let msg = QueryMsg::GetCw20Balance {
        cw20_contract: cw20_contract.to_string(),
        address: ADMIN_CW20.to_string(),
    };
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&coin(0, &cw20_contract))?);

    // If address doesn't exist, return coin with zero amount
    let msg = QueryMsg::GetCw20Balance {
        cw20_contract: cw20_contract.to_string(),
        address: ANOTHER.to_string(),
    };
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&coin(0, &cw20_contract))?);

    // Error if called wrong cw20_contract
    let msg = QueryMsg::GetCw20Balance {
        cw20_contract: contract_addr.to_string(),
        address: ANYONE.to_string(),
    };
    let res: StdResult<QueryResponse> = app.wrap().query_wasm_smart(contract_addr, &msg);
    assert!(res.is_err());

    Ok(())
}
