use cosmwasm_std::{to_binary, Addr, StdResult};
use cw721_base::MintMsg;
use cw_multi_test::Executor;
use mod_sdk::types::QueryResponse;

use crate::{
    msg::QueryMsg,
    tests::helpers::{proper_instantiate, ADMIN, ADMIN_CW721, ANYONE, URI},
};

#[test]
fn test_has_nft() -> StdResult<()> {
    let (mut app, contract_addr, cw721_contract) = proper_instantiate();

    // Mint two tokens
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

    let mint_msg: cw721_base::ExecuteMsg<std::option::Option<std::string::String>, &str> =
        cw721_base::ExecuteMsg::Mint(MintMsg::<Option<String>> {
            token_id: "croncat2".to_string(),
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

    // Return true if the address owns some tokens
    let msg = QueryMsg::AddrHasNft {
        address: ANYONE.to_string(),
        nft_address: cw721_contract.to_string(),
    };
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);
    assert_eq!(
        res.data,
        to_binary(&vec!["croncat".to_string(), "croncat2".to_string()])?
    );

    // Return false if the address doesn't own any tokens on this contract
    let msg = QueryMsg::AddrHasNft {
        address: ADMIN.to_string(),
        nft_address: cw721_contract.to_string(),
    };
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);
    assert_eq!(res.data, to_binary::<std::vec::Vec<String>>(&vec![])?);

    // Wrong nft_address
    let msg = QueryMsg::AddrHasNft {
        address: ANYONE.to_string(),
        nft_address: contract_addr.to_string(),
    };
    let err: StdResult<QueryResponse> = app.wrap().query_wasm_smart(contract_addr.clone(), &msg);
    assert!(err.is_err());

    Ok(())
}
