use cosmwasm_std::{coin, coins, Addr, Empty};
use cw20::Cw20Coin;
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

use crate::msg::InstantiateMsg;

pub const ADMIN: &str = "cosmos1sjllsnramtg3ewxqwwrwjxfgc4n4ef9u0tvx7u";
pub const ANYONE: &str = "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
pub const ADMIN_CW20: &str = "cosmos1a7uhnpqthunr2rzj0ww0hwurpn42wyun6c5puz";
pub const ANOTHER: &str = "cosmos1wze8mn5nsgl9qrgazq6a92fvh7m5e6psjcx2du";
pub const NATIVE_DENOM: &str = "atom";

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

fn mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        let accounts: Vec<(u128, String)> = vec![
            (6_000_000, ADMIN.to_string()),
            (6_000_000, ADMIN_CW20.to_string()),
            (1_000_000, ANYONE.to_string()),
        ];
        for (amt, address) in accounts.into_iter() {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(address),
                    vec![coin(amt, NATIVE_DENOM.to_string())],
                )
                .unwrap();
        }
    })
}

pub fn proper_instantiate() -> (App, Addr, Addr) {
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
