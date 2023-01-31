use cosmwasm_std::{Addr, Empty};
use cw20::Cw20Coin;
use cw4::Member;
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use crate::msg::InstantiateMsg;

pub const CREATOR_ADDR: &str = "creator";
const VERSION: &str = "0.1";

fn contract_template() -> Box<dyn Contract<Empty>> {
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

fn cw20_template() -> Box<dyn Contract<Empty>> {
    let cw20 = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(cw20)
}

pub(crate) fn proper_instantiate() -> (App, Addr, Addr, Addr) {
    let mut app = App::default();
    let code_id = app.store_code(contract_template());
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

    (app, contract_addr, cw4_addr, cw20_addr)
}
