use cosmwasm_std::{Addr, Empty};
use cw20::Cw20Coin;
use cw4::Member;
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use crate::msg::InstantiateMsg;

// pub const ADMIN: &str = "cosmos1sjllsnramtg3ewxqwwrwjxfgc4n4ef9u0tvx7u";
pub const ANYONE: &str = "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
// pub const ADMIN_CW20: &str = "cosmos1a7uhnpqthunr2rzj0ww0hwurpn42wyun6c5puz";
// pub const ANOTHER: &str = "cosmos1wze8mn5nsgl9qrgazq6a92fvh7m5e6psjcx2du";
pub const NATIVE_DENOM: &str = "atom";
pub const VERSION: &str = "0.1";

pub const CREATOR_ADDR: &str = "creator";

fn contract_template() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub(crate) fn mod_balances_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        croncat_mod_balances::contract::execute,
        croncat_mod_balances::contract::instantiate,
        croncat_mod_balances::contract::query,
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

fn cw20_template() -> Box<dyn Contract<Empty>> {
    let cw20 = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(cw20)
}

pub(crate) fn proper_instantiate() -> (App, Addr, Addr, Addr, Addr) {
    let mut app = App::default();
    let code_id = app.store_code(contract_template());
    let balances_id = app.store_code(mod_balances_contract());
    let cw4_id = app.store_code(cw4_contract());
    let cw20_id = app.store_code(cw20_template());

    let instantiate = InstantiateMsg {
        version: Some(VERSION.to_owned()),
    };
    let contract_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(CREATOR_ADDR),
            &instantiate,
            &[],
            "mod-generic",
            None,
        )
        .unwrap();
    let balances_addr = app
        .instantiate_contract(
            balances_id,
            Addr::unchecked(CREATOR_ADDR),
            &instantiate,
            &[],
            "mod-balances",
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

    (app, contract_addr, cw4_addr, cw20_addr, balances_addr)
}
