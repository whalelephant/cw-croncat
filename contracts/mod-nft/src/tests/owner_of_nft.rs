use cosmwasm_std::{Addr, StdResult};
use cw721_base::MintMsg;
use cw_multi_test::Executor;
use mod_sdk::types::QueryResponse;

use crate::{
    msg::QueryMsg,
    tests::helpers::{proper_instantiate, ADMIN, ADMIN_CW721, ANYONE, URI},
    types::OwnerOfNft,
};

#[test]
fn test_owner_of_nft() -> StdResult<()> {
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

    // Check that ANYONE is the owner
    let msg = QueryMsg::OwnerOfNft(OwnerOfNft {
        address: ANYONE.to_string(),
        nft_address: cw721_contract.to_string(),
        token_id: "croncat".to_string(),
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(res.result);

    // Return false if it's not the owner
    let msg = QueryMsg::OwnerOfNft(OwnerOfNft {
        address: ADMIN.to_string(),
        nft_address: cw721_contract.to_string(),
        token_id: "croncat".to_string(),
    });
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg)
        .unwrap();
    assert!(!res.result);

    // Wrong token_id
    let msg = QueryMsg::OwnerOfNft(OwnerOfNft {
        address: ANYONE.to_string(),
        nft_address: cw721_contract.to_string(),
        token_id: "croncat2".to_string(),
    });
    let err: StdResult<QueryResponse> = app.wrap().query_wasm_smart(contract_addr.clone(), &msg);
    assert!(err.is_err());

    // Wrong nft_address
    let msg = QueryMsg::OwnerOfNft(OwnerOfNft {
        address: ANYONE.to_string(),
        nft_address: contract_addr.to_string(),
        token_id: "croncat".to_string(),
    });
    let err: StdResult<QueryResponse> = app.wrap().query_wasm_smart(contract_addr, &msg);
    assert!(err.is_err());

    Ok(())
}
