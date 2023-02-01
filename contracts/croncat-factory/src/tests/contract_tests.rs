use cosmwasm_std::{to_binary, Addr, StdError, Uint128};
use croncat_sdk_factory::msg::{
    Config, ContractMetadataResponse, EntryResponse, ModuleInstantiateInfo, VersionKind,
};
use cw20::Cw20Coin;
use cw_multi_test::Executor;

use super::{contracts, helpers::default_app, ADMIN, ANYONE, DENOM};
use crate::{msg::*, ContractError};

#[test]
fn successful_inits() {
    let mut app = default_app();
    let contract_code_id = app.store_code(contracts::croncat_factory_contract());

    let init_msg = InstantiateMsg { owner_addr: None };

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

    let config: Config = app
        .wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::Config {})
        .unwrap();
    assert_eq!(config.owner_addr, Addr::unchecked(ADMIN));

    let init_msg = InstantiateMsg {
        owner_addr: Some(ANYONE.to_owned()),
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

    let config: Config = app
        .wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::Config {})
        .unwrap();
    assert_eq!(config.owner_addr, Addr::unchecked(ANYONE));
}

#[test]
fn failure_inits() {
    let mut app = default_app();
    let contract_code_id = app.store_code(contracts::croncat_factory_contract());

    let init_msg = InstantiateMsg {
        owner_addr: Some("InVaLidAdDrEsS".to_owned()),
    };

    let err: ContractError = app
        .instantiate_contract(
            contract_code_id,
            Addr::unchecked(ADMIN),
            &init_msg,
            &[],
            "factory",
            None,
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        ContractError::Std(StdError::generic_err(
            "Invalid input: address not normalized"
        ))
    );
}

#[test]
fn deploy_check() {
    let mut app = default_app();
    let contract_code_id = app.store_code(contracts::croncat_factory_contract());
    let cw20_code_id = app.store_code(contracts::cw20_contract());

    let init_msg = InstantiateMsg {
        owner_addr: Some(ADMIN.to_owned()),
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
        contract_name: "manager".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::Deploy {
            kind: VersionKind::Manager,
            module_instantiate_info: manager_module_instantiate_info,
        },
        &[],
    )
    .unwrap();

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
        contract_name: "tasks".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::Deploy {
            kind: VersionKind::Tasks,
            module_instantiate_info: tasks_module_instantiate_info,
        },
        &[],
    )
    .unwrap();

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
        contract_name: "agents".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::Deploy {
            kind: VersionKind::Agents,
            module_instantiate_info: agents_module_instantiate_info,
        },
        &[],
    )
    .unwrap();
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
        contract_name: "library".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::Deploy {
            kind: VersionKind::Library,
            module_instantiate_info: library_module_instantiate_info,
        },
        &[],
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
    let _manager_metadata = manager_metadatas.remove(0);
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
    let _tasks_metadata = tasks_metadatas.remove(0);
    // TODO check it's tasks

    let mut agents_metadatas: Vec<ContractMetadataResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::VersionsByContractName {
                contract_name: "agents".to_string(),
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(agents_metadatas.len(), 1);
    let _agents_metadata = agents_metadatas.remove(0);

    // TODO check it is agents
}

#[test]
fn failure_deploy() {
    let mut app = default_app();
    let contract_code_id = app.store_code(contracts::croncat_factory_contract());
    let cw20_code_id = app.store_code(contracts::cw20_contract());

    let init_msg = InstantiateMsg {
        owner_addr: Some(ADMIN.to_owned()),
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
        contract_name: "manager".to_owned(),
    };

    // Not a owner_addr
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &ExecuteMsg::Deploy {
                kind: VersionKind::Manager,
                module_instantiate_info: manager_module_instantiate_info,
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, ContractError::Unauthorized {});

    let bad_module_instantiate_info = ModuleInstantiateInfo {
        code_id: cw20_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: Default::default(),
        contract_name: "manager".to_owned(),
    };

    // Not a wrong_addr
    let err: StdError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr,
            &ExecuteMsg::Deploy {
                kind: VersionKind::Manager,
                module_instantiate_info: bad_module_instantiate_info,
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        err,
        StdError::ParseErr {
            target_type: "cw20_base::msg::InstantiateMsg".to_owned(),
            msg: "EOF while parsing a JSON value.".to_owned()
        }
    )
}

#[test]
fn update_config() {
    let mut app = default_app();
    let contract_code_id = app.store_code(contracts::croncat_factory_contract());

    let init_msg = InstantiateMsg {
        owner_addr: Some(ADMIN.to_owned()),
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

    let update_config_msg = ExecuteMsg::UpdateConfig {
        owner_addr: ANYONE.to_owned(),
    };

    // Not owner_addr execution
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &update_config_msg,
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, ContractError::Unauthorized {});

    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &update_config_msg,
        &[],
    )
    .unwrap();

    let new_config: Config = app
        .wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::Config {})
        .unwrap();
    assert_eq!(
        new_config,
        Config {
            owner_addr: Addr::unchecked(ANYONE)
        }
    )
}

#[test]
fn remove() {
    let mut app = default_app();
    let contract_code_id = app.store_code(contracts::croncat_factory_contract());
    let cw20_code_id = app.store_code(contracts::cw20_contract());

    let init_msg = InstantiateMsg {
        owner_addr: Some(ADMIN.to_owned()),
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
        contract_name: "library".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::Deploy {
            kind: VersionKind::Library,
            module_instantiate_info: library_module_instantiate_info,
        },
        &[],
    )
    .unwrap();
    let library_v2_module_instantiate_info = ModuleInstantiateInfo {
        code_id: cw20_code_id,
        version: [0, 2],
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
        contract_name: "library".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::Deploy {
            kind: VersionKind::Library,
            module_instantiate_info: library_v2_module_instantiate_info,
        },
        &[],
    )
    .unwrap();

    // not owner_addr
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &ExecuteMsg::Remove {
                contract_name: "library".to_owned(),
                version: [0, 1],
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, ContractError::Unauthorized {});

    // Unknown contract removed
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &ExecuteMsg::Remove {
                contract_name: "manager".to_owned(),
                version: [0, 2],
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, ContractError::UnknownContract {});

    // Latest version can't get removed
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &ExecuteMsg::Remove {
                contract_name: "library".to_owned(),
                version: [0, 2],
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, ContractError::LatestVersionRemove {});

    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::Remove {
            contract_name: "library".to_owned(),
            version: [0, 1],
        },
        &[],
    )
    .unwrap();

    let all_entries: Vec<EntryResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::AllEntries {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(all_entries.len(), 1);
    assert_eq!(all_entries[0].metadata.version, [0, 2]);
}

#[test]
fn remove_paused_checks() {
    let mut app = default_app();
    let contract_code_id = app.store_code(contracts::croncat_factory_contract());

    let init_msg = InstantiateMsg {
        owner_addr: Some(ADMIN.to_owned()),
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

    let manager_id = app.store_code(contracts::croncat_manager_contract());
    let manager_init_msg = croncat_manager::msg::InstantiateMsg {
        denom: DENOM.to_owned(),
        version: Some("0.1".to_owned()),
        croncat_tasks_key: ("tasks".to_owned(), [0, 1]),
        croncat_agents_key: ("agents".to_owned(), [0, 1]),
        owner_addr: Some(ADMIN.to_owned()),
        gas_price: None,
        treasury_addr: None,
    };
    // Deploy first version of the contract
    let manager_contract_instantiate_info = ModuleInstantiateInfo {
        code_id: manager_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&manager_init_msg).unwrap(),
        contract_name: "manager".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::Deploy {
            kind: VersionKind::Manager,
            module_instantiate_info: manager_contract_instantiate_info,
        },
        &[],
    )
    .unwrap();
    // Deploy the second version of the contract
    let manager_v2_contract_instantiate_info = ModuleInstantiateInfo {
        code_id: manager_id,
        version: [0, 2],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&manager_init_msg).unwrap(),
        contract_name: "manager".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::Deploy {
            kind: VersionKind::Manager,
            module_instantiate_info: manager_v2_contract_instantiate_info,
        },
        &[],
    )
    .unwrap();

    // Not paused by default
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &ExecuteMsg::Remove {
                contract_name: "manager".to_owned(),
                version: [0, 1],
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ContractError::NotPaused {});
    let manager_metadatas: Vec<ContractMetadataResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::VersionsByContractName {
                contract_name: "manager".to_owned(),
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    let manager_addr = manager_metadatas
        .into_iter()
        .find(|metadata| metadata.version == [0, 1])
        .map(|metadata| metadata.contract_addr)
        .unwrap();
    // make it paused
    app.execute_contract(
        Addr::unchecked(ADMIN),
        manager_addr,
        &croncat_manager::msg::ExecuteMsg::UpdateConfig(Box::new(
            croncat_sdk_manager::types::UpdateConfig {
                owner_addr: None,
                paused: Some(true),
                agent_fee: None,
                treasury_fee: None,
                gas_price: None,
                croncat_tasks_key: None,
                croncat_agents_key: None,
                treasury_addr: None,
            },
        )),
        &[],
    )
    .unwrap();

    // remove it after it got paused
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::Remove {
            contract_name: "manager".to_owned(),
            version: [0, 1],
        },
        &[],
    )
    .unwrap();
    // Make sure it's gone
    let manager_metadatas: Vec<ContractMetadataResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::VersionsByContractName {
                contract_name: "manager".to_owned(),
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    let manager_versions: Vec<[u8; 2]> = manager_metadatas
        .into_iter()
        .map(|metadata| metadata.version)
        .collect();
    // only last version left
    assert_eq!(manager_versions, vec![[0,2]]);
}

#[test]
fn update_metadata() {
    let mut app = default_app();
    let contract_code_id = app.store_code(contracts::croncat_factory_contract());
    let cw20_code_id = app.store_code(contracts::cw20_contract());

    let init_msg = InstantiateMsg {
        owner_addr: Some(ADMIN.to_owned()),
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
        contract_name: "library".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::Deploy {
            kind: VersionKind::Library,
            module_instantiate_info: library_module_instantiate_info,
        },
        &[],
    )
    .unwrap();

    // Not owner_addr
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &ExecuteMsg::UpdateMetadata {
                contract_name: "library".to_owned(),
                version: [0, 1],
                changelog_url: Some("new changelog".to_owned()),
                schema: None,
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, ContractError::Unauthorized {});

    // Wrong contract_name
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &ExecuteMsg::UpdateMetadata {
                contract_name: "manager".to_owned(),
                version: [0, 1],
                changelog_url: Some("new changelog".to_owned()),
                schema: None,
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, ContractError::UnknownContract {});

    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::UpdateMetadata {
            contract_name: "library".to_owned(),
            version: [0, 1],
            changelog_url: Some("new changelog".to_owned()),
            schema: None,
        },
        &[],
    )
    .unwrap();

    let metadatas: Vec<ContractMetadataResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::VersionsByContractName {
                contract_name: "library".to_owned(),
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(metadatas[0].changelog_url, Some("new changelog".to_owned()));
    assert_eq!(metadatas[0].schema, None);

    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::UpdateMetadata {
            contract_name: "library".to_owned(),
            version: [0, 1],
            changelog_url: None,
            schema: Some("new schema".to_owned()),
        },
        &[],
    )
    .unwrap();

    let metadata: Option<ContractMetadataResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::LatestContract {
                contract_name: "library".to_owned(),
            },
        )
        .unwrap();
    let metadata = metadata.unwrap();
    assert_eq!(metadata.changelog_url, Some("new changelog".to_owned()));
    assert_eq!(metadata.schema, Some("new schema".to_owned()));
}
