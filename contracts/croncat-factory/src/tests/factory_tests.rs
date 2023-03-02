use cosmwasm_std::{to_binary, Addr, Binary, StdError, WasmMsg};
use croncat_sdk_core::types::GasPrice;
use croncat_sdk_factory::msg::{
    Config, ContractMetadataInfo, ContractMetadataResponse, EntryResponse, FactoryExecuteMsg,
    ModuleInstantiateInfo, VersionKind,
};
use cw_multi_test::Executor;

use super::{contracts, helpers::default_app, ADMIN, AGENT2, ANYONE, DENOM};
use crate::{msg::*, tests::PARTICIPANT0, ContractError};

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
    let factory_code_id = app.store_code(contracts::croncat_factory_contract());
    let manager_code_id = app.store_code(contracts::croncat_manager_contract());
    let agents_code_id = app.store_code(contracts::croncat_agents_contract());
    let tasks_code_id = app.store_code(contracts::croncat_tasks_contract());
    let mod_balances_code_id = app.store_code(contracts::croncat_mod_balances());

    let init_msg = InstantiateMsg {
        owner_addr: Some(ADMIN.to_owned()),
    };
    let contract_addr = app
        .instantiate_contract(
            factory_code_id,
            Addr::unchecked(ADMIN),
            &init_msg,
            &[],
            "factory",
            None,
        )
        .unwrap();
    let manager_module_instantiate_info = ModuleInstantiateInfo {
        code_id: manager_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&croncat_manager::msg::InstantiateMsg {
            denom: "cron".to_owned(),
            version: Some("0.1".to_owned()),
            croncat_tasks_key: ("tasks".to_owned(), [0, 1]),
            croncat_agents_key: ("agents".to_owned(), [0, 1]),
            owner_addr: Some(ANYONE.to_owned()),
            gas_price: Some(GasPrice {
                numerator: 10,
                denominator: 20,
                gas_adjustment_numerator: 30,
            }),
            treasury_addr: Some(AGENT2.to_owned()),
            cw20_whitelist: Some(vec![PARTICIPANT0.to_owned()]),
        })
        .unwrap(),
        contract_name: "manager".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &FactoryExecuteMsg::Deploy {
            kind: VersionKind::Manager,
            module_instantiate_info: manager_module_instantiate_info,
        },
        &[],
    )
    .unwrap();

    let tasks_module_instantiate_info = ModuleInstantiateInfo {
        code_id: tasks_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&croncat_tasks::msg::InstantiateMsg {
            chain_name: "cron".to_owned(),
            version: Some("0.1".to_owned()),
            owner_addr: Some(ANYONE.to_owned()),
            croncat_manager_key: ("definitely_not_manager".to_owned(), [4, 2]),
            croncat_agents_key: ("definitely_not_agents".to_owned(), [42, 0]),
            slot_granularity_time: Some(10),
            gas_base_fee: Some(1),
            gas_action_fee: Some(2),
            gas_query_fee: Some(3),
            gas_limit: Some(10),
        })
        .unwrap(),
        contract_name: "tasks".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &FactoryExecuteMsg::Deploy {
            kind: VersionKind::Tasks,
            module_instantiate_info: tasks_module_instantiate_info,
        },
        &[],
    )
    .unwrap();

    let agents_module_instantiate_info = ModuleInstantiateInfo {
        code_id: agents_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&croncat_agents::msg::InstantiateMsg {
            version: Some("0.1".to_owned()),
            croncat_manager_key: ("manager".to_owned(), [0, 1]),
            croncat_tasks_key: ("tasks".to_owned(), [0, 1]),
            owner_addr: Some(ADMIN.to_owned()),
            min_coins_for_agent_registration: None,
            agent_nomination_duration: None,
            min_tasks_per_agent: None,
            agents_eject_threshold: None,
            min_active_agent_count: None,
        })
        .unwrap(),
        contract_name: "agents".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &FactoryExecuteMsg::Deploy {
            kind: VersionKind::Agents,
            module_instantiate_info: agents_module_instantiate_info,
        },
        &[],
    )
    .unwrap();
    let library_module_instantiate_info = ModuleInstantiateInfo {
        code_id: mod_balances_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&croncat_mod_balances::msg::InstantiateMsg {
            version: Some("0.1".to_owned()),
        })
        .unwrap(),
        contract_name: "library".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &FactoryExecuteMsg::Deploy {
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

    let mut manager_metadatas: Vec<ContractMetadataInfo> = app
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
    // check it's manager
    assert_eq!(
        manager_metadata.kind,
        VersionKind::Manager,
        "Not manager contract"
    );

    let mut tasks_metadatas: Vec<ContractMetadataInfo> = app
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
    // check it's tasks
    assert_eq!(
        tasks_metadata.kind,
        VersionKind::Tasks,
        "Not tasks contract"
    );

    let mut agents_metadatas: Vec<ContractMetadataInfo> = app
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
    let agents_metadata = agents_metadatas.remove(0);
    // check it is agents
    assert_eq!(
        agents_metadata.kind,
        VersionKind::Agents,
        "Not agents contract"
    );
}

#[test]
fn failure_deploy() {
    let mut app = default_app();
    let contract_code_id = app.store_code(contracts::croncat_factory_contract());
    let manager_code_id = app.store_code(contracts::croncat_manager_contract());

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
        code_id: manager_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&croncat_manager::msg::InstantiateMsg {
            denom: "cron".to_owned(),
            version: Some("0.1".to_owned()),
            croncat_tasks_key: ("tasks".to_owned(), [0, 1]),
            croncat_agents_key: ("agents".to_owned(), [0, 1]),
            owner_addr: Some(ANYONE.to_owned()),
            gas_price: Some(GasPrice {
                numerator: 10,
                denominator: 20,
                gas_adjustment_numerator: 30,
            }),
            treasury_addr: Some(AGENT2.to_owned()),
            cw20_whitelist: Some(vec![PARTICIPANT0.to_owned()]),
        })
        .unwrap(),
        contract_name: "manager".to_owned(),
    };

    // Not a owner_addr
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &FactoryExecuteMsg::Deploy {
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
        code_id: manager_code_id,
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
            contract_addr.clone(),
            &FactoryExecuteMsg::Deploy {
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
            target_type: "croncat_sdk_manager::msg::ManagerInstantiateMsg".to_owned(),
            msg: "EOF while parsing a JSON value.".to_owned()
        }
    );

    // Test to make sure deploys can't be overwritten
    let manager_module_instantiate_info_2 = ModuleInstantiateInfo {
        code_id: manager_code_id,
        version: [0, 2],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&croncat_manager::msg::InstantiateMsg {
            denom: "cron".to_owned(),
            version: Some("0.1".to_owned()),
            croncat_tasks_key: ("tasks".to_owned(), [0, 1]),
            croncat_agents_key: ("agents".to_owned(), [0, 1]),
            owner_addr: Some(ANYONE.to_owned()),
            gas_price: Some(GasPrice {
                numerator: 10,
                denominator: 20,
                gas_adjustment_numerator: 30,
            }),
            treasury_addr: Some(AGENT2.to_owned()),
            cw20_whitelist: Some(vec![PARTICIPANT0.to_owned()]),
        })
        .unwrap(),
        contract_name: "manager".to_owned(),
    };

    // expect 1 success here
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &FactoryExecuteMsg::Deploy {
            kind: VersionKind::Manager,
            module_instantiate_info: manager_module_instantiate_info_2.clone(),
        },
        &[],
    )
    .expect("first deploy went well thank you");

    // Can't redeploy same version
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr,
            &FactoryExecuteMsg::Deploy {
                kind: VersionKind::Manager,
                module_instantiate_info: manager_module_instantiate_info_2,
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, ContractError::VersionExists {})
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

    let update_config_msg = FactoryExecuteMsg::UpdateConfig {
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
    let mod_balances_code_id = app.store_code(contracts::croncat_mod_balances());

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
        code_id: mod_balances_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&croncat_mod_balances::msg::InstantiateMsg {
            version: Some("0.1".to_owned()),
        })
        .unwrap(),
        contract_name: "library".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &FactoryExecuteMsg::Deploy {
            kind: VersionKind::Library,
            module_instantiate_info: library_module_instantiate_info,
        },
        &[],
    )
    .unwrap();
    let library_v2_module_instantiate_info = ModuleInstantiateInfo {
        code_id: mod_balances_code_id,
        version: [0, 2],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&croncat_mod_balances::msg::InstantiateMsg {
            version: Some("0.1".to_owned()),
        })
        .unwrap(),
        contract_name: "library".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &FactoryExecuteMsg::Deploy {
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
            &FactoryExecuteMsg::Remove {
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
            &FactoryExecuteMsg::Remove {
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
            &FactoryExecuteMsg::Remove {
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
        &FactoryExecuteMsg::Remove {
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
        cw20_whitelist: None,
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
        &FactoryExecuteMsg::Deploy {
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
        &FactoryExecuteMsg::Deploy {
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
            &FactoryExecuteMsg::Remove {
                contract_name: "manager".to_owned(),
                version: [0, 1],
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ContractError::NotPaused {});
    let manager_metadatas: Vec<ContractMetadataInfo> = app
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
                cw20_whitelist: None,
            },
        )),
        &[],
    )
    .unwrap();

    // remove it after it got paused
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &FactoryExecuteMsg::Remove {
            contract_name: "manager".to_owned(),
            version: [0, 1],
        },
        &[],
    )
    .unwrap();
    // Make sure it's gone
    let manager_metadatas: Vec<ContractMetadataInfo> = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
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
    assert_eq!(manager_versions, vec![[0, 2]]);
}

#[test]
fn update_metadata() {
    let mut app = default_app();
    let contract_code_id = app.store_code(contracts::croncat_factory_contract());
    let mod_balances_code_id = app.store_code(contracts::croncat_mod_balances());

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
        code_id: mod_balances_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&croncat_mod_balances::msg::InstantiateMsg {
            version: Some("0.1".to_owned()),
        })
        .unwrap(),
        contract_name: "library".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &FactoryExecuteMsg::Deploy {
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
            &FactoryExecuteMsg::UpdateMetadata {
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
            &FactoryExecuteMsg::UpdateMetadata {
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
        &FactoryExecuteMsg::UpdateMetadata {
            contract_name: "library".to_owned(),
            version: [0, 1],
            changelog_url: Some("new changelog".to_owned()),
            schema: None,
        },
        &[],
    )
    .unwrap();

    let metadatas: Vec<ContractMetadataInfo> = app
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
        &FactoryExecuteMsg::UpdateMetadata {
            contract_name: "library".to_owned(),
            version: [0, 1],
            changelog_url: None,
            schema: Some("new schema".to_owned()),
        },
        &[],
    )
    .unwrap();

    let metadata: ContractMetadataResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::LatestContract {
                contract_name: "library".to_owned(),
            },
        )
        .unwrap();
    let metadata = metadata.metadata.unwrap();
    assert_eq!(metadata.changelog_url, Some("new changelog".to_owned()));
    assert_eq!(metadata.schema, Some("new schema".to_owned()));
}

#[test]
fn fail_and_success_proxy() {
    let mut app = default_app();
    let contract_code_id = app.store_code(contracts::croncat_factory_contract());
    let manager_code_id = app.store_code(contracts::croncat_manager_contract());

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
        code_id: manager_code_id,
        version: [0, 1],
        commit_id: "some".to_owned(),
        checksum: "qwe123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&croncat_manager::msg::InstantiateMsg {
            denom: "cron".to_owned(),
            version: Some("0.1".to_owned()),
            croncat_tasks_key: ("tasks".to_owned(), [0, 1]),
            croncat_agents_key: ("agents".to_owned(), [0, 1]),
            owner_addr: None,
            gas_price: Some(GasPrice {
                numerator: 10,
                denominator: 20,
                gas_adjustment_numerator: 30,
            }),
            treasury_addr: Some(AGENT2.to_owned()),
            cw20_whitelist: Some(vec![PARTICIPANT0.to_owned()]),
        })
        .unwrap(),
        contract_name: "manager".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &FactoryExecuteMsg::Deploy {
            kind: VersionKind::Manager,
            module_instantiate_info: manager_module_instantiate_info,
        },
        &[],
    )
    .unwrap();

    // Get the manager contract_addr
    let manager_metadata: ContractMetadataResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::LatestContract {
                contract_name: "manager".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        manager_metadata.metadata.clone().unwrap().code_id,
        manager_code_id
    );

    let proxy_msg = FactoryExecuteMsg::Proxy {
        msg: WasmMsg::Execute {
            contract_addr: manager_metadata.metadata.unwrap().contract_addr.to_string(),
            msg: to_binary(&croncat_sdk_manager::msg::ManagerExecuteMsg::UpdateConfig(
                Box::new(croncat_sdk_manager::types::UpdateConfig {
                    owner_addr: None,
                    paused: None,
                    agent_fee: None,
                    treasury_fee: Some(10), // simulate moving to 0.01%
                    gas_price: None,
                    croncat_tasks_key: None,
                    croncat_agents_key: None,
                    treasury_addr: None,
                    cw20_whitelist: None,
                }),
            ))
            .unwrap(),
            funds: vec![],
        },
    };

    let bad_msg_proxy_msg = FactoryExecuteMsg::Proxy {
        msg: WasmMsg::Instantiate {
            admin: Some(contract_addr.to_string()),
            code_id: manager_code_id,
            msg: Binary::default(),
            funds: vec![],
            label: "bad msg, bad".to_string(),
        },
    };

    let bad_version_proxy_msg = FactoryExecuteMsg::Proxy {
        msg: WasmMsg::Execute {
            contract_addr: Addr::unchecked(ANYONE).to_string(),
            msg: to_binary(&croncat_sdk_manager::msg::ManagerExecuteMsg::UpdateConfig(
                Box::new(croncat_sdk_manager::types::UpdateConfig {
                    owner_addr: None,
                    paused: None,
                    agent_fee: None,
                    treasury_fee: Some(10), // simulate moving to 0.01%
                    gas_price: None,
                    croncat_tasks_key: None,
                    croncat_agents_key: None,
                    treasury_addr: None,
                    cw20_whitelist: None,
                }),
            ))
            .unwrap(),
            funds: vec![],
        },
    };

    // Not the factory owner
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &proxy_msg,
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ContractError::Unauthorized {});

    // Not valid msg
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &bad_msg_proxy_msg,
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ContractError::UnknownMethod {});

    // Not a valid factory version contract
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &bad_version_proxy_msg,
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ContractError::UnknownContract {});

    // Okay yasssss ill let you work
    let res = app
        .execute_contract(Addr::unchecked(ADMIN), contract_addr, &proxy_msg, &[])
        .unwrap();
    // Check for action proxy & action update_config
    assert_eq!(res.events[1].attributes[1].value, "proxy");
    assert_eq!(res.events[3].attributes[1].value, "update_config");
}
