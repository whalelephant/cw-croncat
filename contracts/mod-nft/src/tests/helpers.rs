use cosmwasm_std::{coin, coins, Addr, Empty};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

use crate::msg::InstantiateMsg;

fn contract_template() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

fn cw721_template() -> Box<dyn Contract<Empty>> {
    let cw721 = ContractWrapper::new(
        cw721_base::entry::execute,
        cw721_base::entry::instantiate,
        cw721_base::entry::query,
    );
    Box::new(cw721)
}

pub const ADMIN: &str = "cosmos1sjllsnramtg3ewxqwwrwjxfgc4n4ef9u0tvx7u";
pub const ANYONE: &str = "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
pub const ADMIN_CW721: &str = "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
const NATIVE_DENOM: &str = "atom";
pub const URI: &str = "https://testnet.daodao.zone/dao/juno1p2fuar474uv2p6vfnr2eu4nv9rx4m0qd26xuygkesje0r6pzhrssvrnh2y";

fn mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        let accounts: Vec<(u128, String)> = vec![
            (6_000_000, ADMIN.to_string()),
            (6_000_000, ADMIN_CW721.to_string()),
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

pub(crate) fn proper_instantiate() -> (App, Addr, Addr) {
    let mut app = mock_app();
    let cw_template_id = app.store_code(contract_template());
    let owner_addr = Addr::unchecked(ADMIN);
    let nft_owner_addr = Addr::unchecked(ADMIN_CW721);
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

    let cw721_id = app.store_code(cw721_template());
    let msg = cw721_base::msg::InstantiateMsg {
        name: "Name".to_string(),
        symbol: "Symbol".to_string(),
        minter: ADMIN_CW721.to_string(),
    };
    let cw721_addr = app
        .instantiate_contract(cw721_id, nft_owner_addr, &msg, &[], "Fungible-tokens", None)
        .unwrap();
    (app, cw_template_contract_addr, cw721_addr)
}
