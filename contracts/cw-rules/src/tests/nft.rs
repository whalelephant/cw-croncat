use cosmwasm_std::{coin, coins, Addr, Binary, Empty, StdResult};
use cw721_base::MintMsg;
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use cw_rules_core::types::CheckOwnerOfNft;

use cw_rules_core::msg::{InstantiateMsg, QueryMsg, RuleResponse};

pub fn contract_template() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub fn cw721_template() -> Box<dyn Contract<Empty>> {
    let cw721 = ContractWrapper::new(
        cw721_base::entry::execute,
        cw721_base::entry::instantiate,
        cw721_base::entry::query,
    );
    Box::new(cw721)
}

const ADMIN: &str = "cosmos1sjllsnramtg3ewxqwwrwjxfgc4n4ef9u0tvx7u";
const ANYONE: &str = "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
const ADMIN_CW721: &str = "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
const NATIVE_DENOM: &str = "atom";
const URI: &str = "https://testnet.daodao.zone/dao/juno1p2fuar474uv2p6vfnr2eu4nv9rx4m0qd26xuygkesje0r6pzhrssvrnh2y";

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

fn proper_instantiate() -> (App, Addr, Addr) {
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

#[test]
fn test_check_owner_nft() -> StdResult<()> {
    let (mut app, contract_addr, cw721_contract) = proper_instantiate();

    let mint_msg: cw721_base::ExecuteMsg<std::option::Option<std::string::String>, &str> =
        cw721_base::ExecuteMsg::Mint(MintMsg::<Option<String>> {
            token_id: "croncat".to_string(),
            owner: ANYONE.to_string(),
            token_uri: Some(URI.to_string()),
            extension: None,
        });
    app.execute_contract(
        Addr::unchecked(ADMIN_CW721),
        cw721_contract.clone(),
        &mint_msg,
        &[],
    )
    .unwrap();

    let msg = QueryMsg::CheckOwnerOfNft(CheckOwnerOfNft {
        address: ANYONE.to_string(),
        nft_address: cw721_contract.to_string(),
        token_id: "croncat".to_string(),
    });
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.0);

    // Return false if it's a the owner
    let msg = QueryMsg::CheckOwnerOfNft(CheckOwnerOfNft {
        address: ADMIN.to_string(),
        nft_address: cw721_contract.to_string(),
        token_id: "croncat".to_string(),
    });
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.0);

    // Wrong token_id
    let msg = QueryMsg::CheckOwnerOfNft(CheckOwnerOfNft {
        address: ANYONE.to_string(),
        nft_address: cw721_contract.to_string(),
        token_id: "croncat2".to_string(),
    });
    let err: StdResult<RuleResponse<Option<Binary>>> =
        app.wrap().query_wasm_smart(contract_addr, &msg);
    assert!(err.is_err());

    Ok(())
}
