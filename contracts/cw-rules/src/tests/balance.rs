use cosmwasm_std::{coin, coins, to_binary, Addr, Binary, Empty, StdResult, Uint128};
use cw20::{Balance, Cw20Coin, Cw20CoinVerified};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use cw_utils::NativeBalance;

use crate::msg::{InstantiateMsg, QueryMsg, RuleResponse};

pub fn contract_template() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
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

const ADMIN: &str = "cosmos1sjllsnramtg3ewxqwwrwjxfgc4n4ef9u0tvx7u";
const ANYONE: &str = "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
const ADMIN_CW20: &str = "cosmos1a7uhnpqthunr2rzj0ww0hwurpn42wyun6c5puz";
const ANOTHER: &str = "cosmos1wze8mn5nsgl9qrgazq6a92fvh7m5e6psjcx2du";
const NATIVE_DENOM: &str = "atom";

fn mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        let accounts: Vec<(u128, String)> = vec![
            (6_000_000, ADMIN.to_string()),
            (6_000_000, ADMIN_CW20.to_string()),
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
fn test_get_balance() -> StdResult<()> {
    let (app, contract_addr, _) = proper_instantiate();

    let msg = QueryMsg::GetBalance {
        address: ANYONE.to_string(),
        denom: NATIVE_DENOM.to_string(),
    };
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.0);
    assert_eq!(res.1.unwrap(), to_binary(&coin(1000000, NATIVE_DENOM))?);

    // Balance with another denom is zero
    let msg = QueryMsg::GetBalance {
        address: ANYONE.to_string(),
        denom: "juno".to_string(),
    };
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.0);
    assert_eq!(res.1.unwrap(), to_binary(&coin(0, "juno"))?);

    // Address doesn't exist, return zero balance
    let msg = QueryMsg::GetBalance {
        address: ANOTHER.to_string(),
        denom: NATIVE_DENOM.to_string(),
    };
    let res: RuleResponse<Option<Binary>> =
        app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
    assert!(res.0);
    assert_eq!(res.1.unwrap(), to_binary(&coin(0, NATIVE_DENOM))?);

    Ok(())
}

#[test]
fn test_get_cw20_balance() -> StdResult<()> {
    let (app, contract_addr, cw20_contract) = proper_instantiate();

    // Return coin
    let msg = QueryMsg::GetCW20Balance {
        cw20_contract: cw20_contract.to_string(),
        address: ANYONE.to_string(),
    };
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.0);
    assert_eq!(res.1.unwrap(), to_binary(&coin(15, &cw20_contract))?);

    // Return coin if balance is zero
    let msg = QueryMsg::GetCW20Balance {
        cw20_contract: cw20_contract.to_string(),
        address: ADMIN_CW20.to_string(),
    };
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.0);
    assert_eq!(res.1.unwrap(), to_binary(&coin(0, &cw20_contract))?);

    // If address doesn't exist, return coin with zero amount
    let msg = QueryMsg::GetCW20Balance {
        cw20_contract: cw20_contract.to_string(),
        address: ANOTHER.to_string(),
    };
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.0);
    assert_eq!(res.1.unwrap(), to_binary(&coin(0, &cw20_contract))?);

    // Error if called wrong cw20_contract
    let msg = QueryMsg::GetCW20Balance {
        cw20_contract: contract_addr.to_string(),
        address: ANYONE.to_string(),
    };
    let res: StdResult<RuleResponse<Option<Binary>>> =
        app.wrap().query_wasm_smart(contract_addr, &msg);
    assert!(res.is_err());

    Ok(())
}

#[test]
fn test_has_balance_native() -> StdResult<()> {
    let (app, contract_addr, _) = proper_instantiate();

    // Return true if address has more coins
    let msg = QueryMsg::HasBalanceGT {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            900_000u128,
            NATIVE_DENOM.to_string(),
        ))),
    };
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.0);
    assert_eq!(res.1, None);

    // Return true if real and required balances are equal
    let msg = QueryMsg::HasBalanceGT {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            1_000_000u128,
            NATIVE_DENOM.to_string(),
        ))),
    };
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.0);
    assert_eq!(res.1, None);

    // Return false if address has less coins
    let msg = QueryMsg::HasBalanceGT {
        address: ANYONE.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(
            1_100_000u128,
            NATIVE_DENOM.to_string(),
        ))),
    };
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.0);
    assert_eq!(res.1, None);

    // Return false if the account doesn't exist and required_balance is not zero
    let msg = QueryMsg::HasBalanceGT {
        address: ANOTHER.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(1u128, NATIVE_DENOM.to_string()))),
    };
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.0);
    assert_eq!(res.1, None);

    // Return true if required balance is zero
    let msg = QueryMsg::HasBalanceGT {
        address: ANOTHER.to_string(),
        required_balance: Balance::Native(NativeBalance(coins(0u128, NATIVE_DENOM.to_string()))),
    };
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.0);
    assert_eq!(res.1, None);

    Ok(())
}

#[test]
fn test_has_balance_cw20() -> StdResult<()> {
    let (app, contract_addr, cw20_contract) = proper_instantiate();

    // Return true if address has more coins
    let msg = QueryMsg::HasBalanceGT {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(14u128),
        }),
    };
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.0);
    assert_eq!(res.1, None);

    // Return true if real and required balances are equal
    let msg = QueryMsg::HasBalanceGT {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(15u128),
        }),
    };
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.0);
    assert_eq!(res.1, None);

    // Return false if address has less coins
    let msg = QueryMsg::HasBalanceGT {
        address: ANYONE.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(16u128),
        }),
    };
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.0);
    assert_eq!(res.1, None);

    // Return false if the account doesn't exist and required_balance is not zero
    let msg = QueryMsg::HasBalanceGT {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::from(2u128),
        }),
    };
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.0);
    assert_eq!(res.1, None);

    // Return true if required balance is zero
    let msg = QueryMsg::HasBalanceGT {
        address: ANOTHER.to_string(),
        required_balance: Balance::Cw20(Cw20CoinVerified {
            address: Addr::unchecked(&cw20_contract),
            amount: Uint128::zero(),
        }),
    };
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.0);
    assert_eq!(res.1, None);

    Ok(())
}
