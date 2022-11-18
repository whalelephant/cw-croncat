use cosmwasm_std::{coin, coins, to_binary, Addr, StdResult, Uint128};
use cw20::{Balance, Cw20CoinVerified};
use cw_rules_core::types::HasBalanceGte;
use cw_utils::NativeBalance;

use cw_rules_core::msg::{QueryMsg, RuleResponse};

use crate::tests::helpers::{proper_instantiate, ANOTHER};

use super::helpers::{ADMIN_CW20, ANYONE, NATIVE_DENOM};

#[test]
fn test_get_balance() -> StdResult<()> {
    let (app, contract_addr, _) = proper_instantiate();

    let msg = QueryMsg::GetBalance {
        address: ANYONE.to_string(),
        denom: NATIVE_DENOM.to_string(),
    };
    let res: RuleResponse = app
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
    let res: RuleResponse = app
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
    let res: RuleResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&coin(0, NATIVE_DENOM))?);

    Ok(())
}

#[test]
fn test_get_cw20_balance() -> StdResult<()> {
    let (app, contract_addr, cw20_contract) = proper_instantiate();

    // Return coin
    let msg = QueryMsg::GetCw20Balance {
        cw20_contract: cw20_contract.to_string(),
        address: ANYONE.to_string(),
    };
    let res: RuleResponse = app
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
    let res: RuleResponse = app
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
    let res: RuleResponse = app
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
    let res: StdResult<RuleResponse> = app.wrap().query_wasm_smart(contract_addr, &msg);
    assert!(res.is_err());

    Ok(())
}

#[test]
fn test_has_balance_native() -> StdResult<()> {
    let (app, contract_addr, _) = proper_instantiate();

    // Return true if address has more coins
    let msg = QueryMsg::HasBalanceGte(HasBalanceGte {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            900_000u128,
            NATIVE_DENOM.to_string(),
        ))),
    });
    let res: RuleResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(
        res.data,
        to_binary(&Balance::from(coins(1_000_000u128, NATIVE_DENOM))).unwrap()
    );

    // Return true if real and required balances are equal
    let msg = QueryMsg::HasBalanceGte(HasBalanceGte {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            1_000_000u128,
            NATIVE_DENOM.to_string(),
        ))),
    });
    let res: RuleResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(
        res.data,
        to_binary(&Balance::from(coins(1_000_000u128, NATIVE_DENOM))).unwrap()
    );

    // Return false if address has less coins
    let msg = QueryMsg::HasBalanceGte(HasBalanceGte {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            1_100_000u128,
            NATIVE_DENOM.to_string(),
        ))),
    });
    let res: RuleResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(
        res.data,
        to_binary(&Balance::from(coins(1_000_000u128, NATIVE_DENOM))).unwrap()
    );

    // Return false if address has zero coins
    let msg = QueryMsg::HasBalanceGte(HasBalanceGte {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(1_000_000u128, "juno".to_string()))),
    });
    let res: RuleResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(
        res.data,
        to_binary(&Balance::from(coins(1_000_000u128, NATIVE_DENOM))).unwrap()
    );

    // Return false if the account doesn't exist and required_balance is not zero
    let msg = QueryMsg::HasBalanceGte(HasBalanceGte {
        address: ANOTHER.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(1u128, NATIVE_DENOM.to_string()))),
    });
    let res: RuleResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(
        res.data,
        to_binary(&Balance::Native(NativeBalance(vec![]))).unwrap()
    );

    // Return true if required balance is zero
    let msg = QueryMsg::HasBalanceGte(HasBalanceGte {
        address: ANOTHER.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(0u128, NATIVE_DENOM.to_string()))),
    });
    let res: RuleResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(
        res.data,
        to_binary(&Balance::Native(NativeBalance(vec![]))).unwrap()
    );

    Ok(())
}

#[test]
fn test_has_balance_cw20() -> StdResult<()> {
    let (app, contract_addr, cw20_contract) = proper_instantiate();

    // Return true if address has more coins
    let msg = QueryMsg::HasBalanceGte(HasBalanceGte {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(14u128),
        }),
    });
    let res: RuleResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(
        res.data,
        to_binary(&Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(15u128),
        }))
        .unwrap()
    );

    // Return true if real and required balances are equal
    let msg = QueryMsg::HasBalanceGte(HasBalanceGte {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(15u128),
        }),
    });
    let res: RuleResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(
        res.data,
        to_binary(&Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(15u128),
        }))
        .unwrap()
    );

    // Return false if address has less coins
    let msg = QueryMsg::HasBalanceGte(HasBalanceGte {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(16u128),
        }),
    });
    let res: RuleResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(
        res.data,
        to_binary(&Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(15u128),
        }))
        .unwrap()
    );

    // Return false if the account doesn't exist and required_balance is not zero
    let msg = QueryMsg::HasBalanceGte(HasBalanceGte {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(2u128),
        }),
    });
    let res: RuleResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(
        res.data,
        to_binary(&Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::zero(),
        }))
        .unwrap()
    );

    // Return true if required balance is zero
    let msg = QueryMsg::HasBalanceGte(HasBalanceGte {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::zero(),
        }),
    });
    let res: RuleResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(
        res.data,
        to_binary(&Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::zero(),
        }))
        .unwrap()
    );

    Ok(())
}
