use cosmwasm_std::{coins, to_binary, Addr, StdResult, Uint128};
use cw20::{Balance, Cw20CoinVerified};
use cw_utils::NativeBalance;
use mod_sdk::types::QueryResponse;

use crate::{
    msg::QueryMsg,
    tests::helpers::{proper_instantiate, ANOTHER},
    types::{BalanceComparator, HasBalanceComparator},
};

use super::helpers::{ANYONE, NATIVE_DENOM};

#[test]
fn test_has_balance_comparator_native_gte() -> StdResult<()> {
    let (app, contract_addr, _) = proper_instantiate();

    // Return true if address has more coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            900_000u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Gte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return true if real and required balances are equal
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            1_000_000u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Gte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return false if address has less coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            1_100_000u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Gte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return false if address has zero coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(1_000_000u128, "juno".to_string()))),
        comparator: BalanceComparator::Gte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return false if the account doesn't exist and required_balance is not zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(1u128, NATIVE_DENOM.to_string()))),
        comparator: BalanceComparator::Gte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return true if the account doesn't exist and required balance is zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(0u128, NATIVE_DENOM.to_string()))),
        comparator: BalanceComparator::Gte,
    });
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());
    Ok(())
}

#[test]
fn test_has_balance_comparator_native_gt() -> StdResult<()> {
    let (app, contract_addr, _) = proper_instantiate();

    // Return true if address has more coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            900_000u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Gt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return false if real and required balances are equal
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            1_000_000u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Gt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return false if address has less coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            1_100_000u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Gt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return false if address has zero coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(1_000_000u128, "juno".to_string()))),
        comparator: BalanceComparator::Gt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return false if the account doesn't exist and required_balance is not zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(1u128, NATIVE_DENOM.to_string()))),
        comparator: BalanceComparator::Gt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return false if the account doesn't exist and required balance is zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(0u128, NATIVE_DENOM.to_string()))),
        comparator: BalanceComparator::Gt,
    });
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());
    Ok(())
}

#[test]
fn test_has_balance_comparator_native_lte() -> StdResult<()> {
    let (app, contract_addr, _) = proper_instantiate();

    // Return true if address has less coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            1_000_001u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Lte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return true if real and required balances are equal
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            1_000_000u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Lte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return false if address has more coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            999_999u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Lte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return true if address has zero coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(1_000_000u128, "juno".to_string()))),
        comparator: BalanceComparator::Lte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return true if the account doesn't exist and required_balance is not zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(1u128, NATIVE_DENOM.to_string()))),
        comparator: BalanceComparator::Lte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return true if the account doesn't exist and required balance is zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(0u128, NATIVE_DENOM.to_string()))),
        comparator: BalanceComparator::Lte,
    });
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());
    Ok(())
}

#[test]
fn test_has_balance_comparator_native_lt() -> StdResult<()> {
    let (app, contract_addr, _) = proper_instantiate();

    // Return true if address has less coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            1_000_001u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Lt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return false if real and required balances are equal
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            1_000_000u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Lt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return false if address has more coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            999_999u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Lt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return true if address has zero coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(1_000_000u128, "juno".to_string()))),
        comparator: BalanceComparator::Lt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return true if the account doesn't exist and required_balance is not zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(1u128, NATIVE_DENOM.to_string()))),
        comparator: BalanceComparator::Lt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return false if the account doesn't exist and required balance is zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(0u128, NATIVE_DENOM.to_string()))),
        comparator: BalanceComparator::Lt,
    });
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());
    Ok(())
}

#[test]
fn test_has_balance_comparator_native_eq() -> StdResult<()> {
    let (app, contract_addr, _) = proper_instantiate();

    // Return false if address has less coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            1_000_001u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Eq,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return true if real and required balances are equal
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            1_000_000u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Eq,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return false if address has more coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            999_999u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Eq,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return false if address has zero coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(1_000_000u128, "juno".to_string()))),
        comparator: BalanceComparator::Eq,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return true if address has zero coins and required_balance is zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(0u128, "juno".to_string()))),
        comparator: BalanceComparator::Eq,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return false if the account doesn't exist and required_balance is not zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(1u128, NATIVE_DENOM.to_string()))),
        comparator: BalanceComparator::Eq,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return true if the account doesn't exist and required balance is zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(0u128, NATIVE_DENOM.to_string()))),
        comparator: BalanceComparator::Eq,
    });
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());
    Ok(())
}

#[test]
fn test_has_balance_comparator_native_ne() -> StdResult<()> {
    let (app, contract_addr, _) = proper_instantiate();

    // Return true if address has less coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            1_000_001u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Ne,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return false if real and required balances are equal
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            1_000_000u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Ne,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return true if address has more coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            999_999u128,
            NATIVE_DENOM.to_string(),
        ))),
        comparator: BalanceComparator::Ne,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::new(1_000_000)).unwrap());

    // Return true if address has zero coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(1_000_000u128, "juno".to_string()))),
        comparator: BalanceComparator::Ne,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return false if address has zero coins and required_balance is zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(0u128, "juno".to_string()))),
        comparator: BalanceComparator::Ne,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return true if the account doesn't exist and required_balance is not zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(1u128, NATIVE_DENOM.to_string()))),
        comparator: BalanceComparator::Ne,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return false if the account doesn't exist and required balance is zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(0u128, NATIVE_DENOM.to_string()))),
        comparator: BalanceComparator::Ne,
    });
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());
    Ok(())
}

#[test]
fn test_has_balance_comparator_cw20_gte() -> StdResult<()> {
    // Instantiate query and cw20 contracts
    // ANYONE's balance is equal to 15
    let (app, contract_addr, cw20_contract) = proper_instantiate();

    // Return true if address has more coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(14u128),
        }),
        comparator: BalanceComparator::Gte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return true if the balance and required balance are equal
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(15u128),
        }),
        comparator: BalanceComparator::Gte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return false if address has less coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(16u128),
        }),
        comparator: BalanceComparator::Gte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return false if the account doesn't exist and required_balance is not zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(2u128),
        }),
        comparator: BalanceComparator::Gte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return true if the account doesn't exist and required balance is zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::zero(),
        }),
        comparator: BalanceComparator::Gte,
    });
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Error if contract address is wrong
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::zero(),
        }),
        comparator: BalanceComparator::Gte,
    });
    let res_opt: StdResult<QueryResponse> = app.wrap().query_wasm_smart(cw20_contract, &msg);
    assert!(res_opt.is_err());

    Ok(())
}

#[test]
fn test_has_balance_comparator_cw20_gt() -> StdResult<()> {
    // Instantiate query and cw20 contracts
    // ANYONE's balance is equal to 15
    let (app, contract_addr, cw20_contract) = proper_instantiate();

    // Return true if address has more coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(14u128),
        }),
        comparator: BalanceComparator::Gt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return false if the balance and required balance are equal
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(15u128),
        }),
        comparator: BalanceComparator::Gt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return false if address has less coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(16u128),
        }),
        comparator: BalanceComparator::Gt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return false if the account doesn't exist and required_balance is not zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(2u128),
        }),
        comparator: BalanceComparator::Gt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return false if the account doesn't exist and required balance is zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::zero(),
        }),
        comparator: BalanceComparator::Gt,
    });
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Error if contract address is wrong
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::zero(),
        }),
        comparator: BalanceComparator::Gt,
    });
    let res_opt: StdResult<QueryResponse> = app.wrap().query_wasm_smart(cw20_contract, &msg);
    assert!(res_opt.is_err());

    Ok(())
}

#[test]
fn test_has_balance_comparator_cw20_lte() -> StdResult<()> {
    // Instantiate query and cw20 contracts
    // ANYONE's balance is equal to 15
    let (app, contract_addr, cw20_contract) = proper_instantiate();

    // Return true if address has less coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(16u128),
        }),
        comparator: BalanceComparator::Lte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return true if the balance and required balance are equal
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(15u128),
        }),
        comparator: BalanceComparator::Lte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return false if address has more coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(14u128),
        }),
        comparator: BalanceComparator::Lte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return true if the account doesn't exist and required_balance is not zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(2u128),
        }),
        comparator: BalanceComparator::Lte,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return true if the account doesn't exist and required balance is zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::zero(),
        }),
        comparator: BalanceComparator::Lte,
    });
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Error if contract address is wrong
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::zero(),
        }),
        comparator: BalanceComparator::Lte,
    });
    let res_opt: StdResult<QueryResponse> = app.wrap().query_wasm_smart(cw20_contract, &msg);
    assert!(res_opt.is_err());

    Ok(())
}

#[test]
fn test_has_balance_comparator_cw20_lt() -> StdResult<()> {
    // Instantiate query and cw20 contracts
    // ANYONE's balance is equal to 15
    let (app, contract_addr, cw20_contract) = proper_instantiate();

    // Return true if address has less coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(16u128),
        }),
        comparator: BalanceComparator::Lt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return false if the balance and required balance are equal
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(15u128),
        }),
        comparator: BalanceComparator::Lt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return false if address has more coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(14u128),
        }),
        comparator: BalanceComparator::Lt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return true if the account doesn't exist and required_balance is not zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(2u128),
        }),
        comparator: BalanceComparator::Lt,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return false if the account doesn't exist and required balance is zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::zero(),
        }),
        comparator: BalanceComparator::Lt,
    });
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Error if contract address is wrong
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::zero(),
        }),
        comparator: BalanceComparator::Lt,
    });
    let res_opt: StdResult<QueryResponse> = app.wrap().query_wasm_smart(cw20_contract, &msg);
    assert!(res_opt.is_err());

    Ok(())
}

#[test]
fn test_has_balance_comparator_cw20_eq() -> StdResult<()> {
    // Instantiate query and cw20 contracts
    // ANYONE's balance is equal to 15
    let (app, contract_addr, cw20_contract) = proper_instantiate();

    // Return false if address has less coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(16u128),
        }),
        comparator: BalanceComparator::Eq,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return true if the balance and required balance are equal
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(15u128),
        }),
        comparator: BalanceComparator::Eq,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return false if address has more coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(14u128),
        }),
        comparator: BalanceComparator::Eq,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return false if the account doesn't exist and required_balance is not zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(2u128),
        }),
        comparator: BalanceComparator::Eq,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return true if the account doesn't exist and required balance is zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::zero(),
        }),
        comparator: BalanceComparator::Eq,
    });
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Error if contract address is wrong
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::zero(),
        }),
        comparator: BalanceComparator::Eq,
    });
    let res_opt: StdResult<QueryResponse> = app.wrap().query_wasm_smart(cw20_contract, &msg);
    assert!(res_opt.is_err());

    Ok(())
}

#[test]
fn test_has_balance_comparator_cw20_ne() -> StdResult<()> {
    // Instantiate query and cw20 contracts
    // ANYONE's balance is equal to 15
    let (app, contract_addr, cw20_contract) = proper_instantiate();

    // Return true if address has less coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(16u128),
        }),
        comparator: BalanceComparator::Ne,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return false if the balance and required balance are equal
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(15u128),
        }),
        comparator: BalanceComparator::Ne,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return true if address has more coins
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(14u128),
        }),
        comparator: BalanceComparator::Ne,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::from(15u128)).unwrap());

    // Return true if the account doesn't exist and required_balance is not zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(2u128),
        }),
        comparator: BalanceComparator::Ne,
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Return false if the account doesn't exist and required balance is zero
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::zero(),
        }),
        comparator: BalanceComparator::Ne,
    });
    let res: QueryResponse = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary(&Uint128::zero()).unwrap());

    // Error if contract address is wrong
    let msg = QueryMsg::HasBalanceComparator(HasBalanceComparator {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::zero(),
        }),
        comparator: BalanceComparator::Ne,
    });
    let res_opt: StdResult<QueryResponse> = app.wrap().query_wasm_smart(cw20_contract, &msg);
    assert!(res_opt.is_err());

    Ok(())
}
