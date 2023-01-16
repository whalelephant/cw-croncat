use cosmwasm_std::{to_binary, Addr, Uint128};
use croncat_sdk_factory::msg::{EntryResponse, ModuleInstantiateInfo, ContractMetadataResponse};
use cw20::Cw20Coin;
use cw_multi_test::Executor;

use super::{contracts, helpers::default_app, ADMIN, ANYONE};
use crate::msg::*;

#[test]
fn successful_init() {
    let mut app = default_app();
    let contract_code_id = app.store_code(contracts::croncat_factory_contract());
    let cw20_code_id = app.store_code(contracts::cw20_contract());

    let manager_module_instantiate_info = ModuleInstantiateInfo {
        code_id: cw20_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&cw20_base::msg::InstantiateMsg {
            name: "cron".to_owned(),
            symbol: "cat".to_owned(),
            decimals: 5,
            initial_balances: vec![Cw20Coin {
                address: ANYONE.to_owned(),
                amount: Uint128::new(150),
            }],
            mint: None,
            marketing: None,
        })
        .unwrap(),
        label: "manager".to_owned(),
    };
    let tasks_module_instantiate_info = ModuleInstantiateInfo {
        code_id: cw20_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&cw20_base::msg::InstantiateMsg {
            name: "cron".to_owned(),
            symbol: "cat".to_owned(),
            decimals: 5,
            initial_balances: vec![Cw20Coin {
                address: ANYONE.to_owned(),
                amount: Uint128::new(150),
            }],
            mint: None,
            marketing: None,
        })
        .unwrap(),
        label: "tasks".to_owned(),
    };
    let agents_module_instantiate_info = ModuleInstantiateInfo {
        code_id: cw20_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&cw20_base::msg::InstantiateMsg {
            name: "cron".to_owned(),
            symbol: "cat".to_owned(),
            decimals: 5,
            initial_balances: vec![Cw20Coin {
                address: ANYONE.to_owned(),
                amount: Uint128::new(150),
            }],
            mint: None,
            marketing: None,
        })
        .unwrap(),
        label: "agents".to_owned(),
    };
    let library_module_instantiate_info = ModuleInstantiateInfo {
        code_id: cw20_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&cw20_base::msg::InstantiateMsg {
            name: "cron".to_owned(),
            symbol: "cat".to_owned(),
            decimals: 5,
            initial_balances: vec![Cw20Coin {
                address: ANYONE.to_owned(),
                amount: Uint128::new(150),
            }],
            mint: None,
            marketing: None,
        })
        .unwrap(),
        label: "library".to_owned(),
    };

    let init_msg = InstantiateMsg {
        owner_addr: Some(ADMIN.to_owned()),
        manager_module_instantiate_info,
        tasks_module_instantiate_info,
        agents_module_instantiate_info,
        library_modules_instantiate_info: vec![library_module_instantiate_info],
    };

    let contract_addr = app
        .instantiate_contract(
            contract_code_id,
            Addr::unchecked(ADMIN),
            &init_msg,
            &[],
            "factory",
            None,
        )
        .unwrap();

    let contracts: Vec<EntryResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::AllEntries {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(contracts.len(), 4);

    let mut manager_metadatas: Vec<ContractMetadataResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::VersionsByContractName {
                contract_name: "manager".to_string(),
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(manager_metadatas.len(), 1);
    let manager_metadata = manager_metadatas.remove(0);
    // TODO check it's manager

    let mut tasks_metadatas: Vec<ContractMetadataResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::VersionsByContractName {
                contract_name: "tasks".to_string(),
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(tasks_metadatas.len(), 1);
    let tasks_metadata = tasks_metadatas.remove(0);
    // TODO check it's tasks

    let mut agents_metadatas: Vec<ContractMetadataResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::VersionsByContractName {
                contract_name: "agents".to_string(),
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(agents_metadatas.len(), 1);
    let agents_metadata = agents_metadatas.remove(0);

    // TODO check it is agents
}

#[test]
fn failure_init() {
    let mut app = default_app();
    let contract_code_id = app.store_code(contracts::croncat_factory_contract());
    let cw20_code_id = app.store_code(contracts::cw20_contract());

    let manager_module_instantiate_info = ModuleInstantiateInfo {
        code_id: cw20_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&cw20_base::msg::InstantiateMsg {
            name: "cron".to_owned(),
            symbol: "cat".to_owned(),
            decimals: 5,
            initial_balances: vec![Cw20Coin {
                address: ANYONE.to_owned(),
                amount: Uint128::new(150),
            }],
            mint: None,
            marketing: None,
        })
        .unwrap(),
        label: "manager".to_owned(),
    };
    let tasks_module_instantiate_info = ModuleInstantiateInfo {
        code_id: cw20_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&cw20_base::msg::InstantiateMsg {
            name: "cron".to_owned(),
            symbol: "cat".to_owned(),
            decimals: 5,
            initial_balances: vec![Cw20Coin {
                address: ANYONE.to_owned(),
                amount: Uint128::new(150),
            }],
            mint: None,
            marketing: None,
        })
        .unwrap(),
        label: "tasks".to_owned(),
    };
    let agents_module_instantiate_info = ModuleInstantiateInfo {
        code_id: cw20_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&cw20_base::msg::InstantiateMsg {
            name: "cron".to_owned(),
            symbol: "cat".to_owned(),
            decimals: 5,
            initial_balances: vec![Cw20Coin {
                address: ANYONE.to_owned(),
                amount: Uint128::new(150),
            }],
            mint: None,
            marketing: None,
        })
        .unwrap(),
        label: "agents".to_owned(),
    };
    let library_module_instantiate_info = ModuleInstantiateInfo {
        code_id: cw20_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        // bad instantiate_msg
        msg: Default::default(),
        label: "library".to_owned(),
    };

    let init_msg = InstantiateMsg {
        owner_addr: Some(ADMIN.to_owned()),
        manager_module_instantiate_info,
        tasks_module_instantiate_info,
        agents_module_instantiate_info,
        library_modules_instantiate_info: vec![library_module_instantiate_info],
    };

    let err = app.instantiate_contract(
        contract_code_id,
        Addr::unchecked(ADMIN),
        &init_msg,
        &[],
        "factory",
        None,
    );
    assert!(err.is_err())
}
