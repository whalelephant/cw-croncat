use cosmwasm_std::{Addr, StdError};
use croncat_sdk_tasks::types::Config;
use cw_multi_test::Executor;

use super::{
    contracts,
    helpers::{default_app, default_instantiate_msg, init_factory, init_manager, init_tasks},
    ADMIN,
};
use crate::{
    contract::{GAS_ACTION_FEE, GAS_BASE_FEE, GAS_LIMIT, GAS_QUERY_FEE, SLOT_GRANULARITY_TIME},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    tests::ANYONE,
    ContractError,
};

mod instantiate_tests {
    use super::*;

    #[test]
    fn default_init() {
        let mut app = default_app();
        let factory_addr = init_factory(&mut app);

        let instantiate_msg: InstantiateMsg = default_instantiate_msg();
        let tasks_addr = init_tasks(&mut app, &instantiate_msg, &factory_addr);
        let config: Config = app
            .wrap()
            .query_wasm_smart(tasks_addr, &QueryMsg::Config {})
            .unwrap();
        let expected_config = Config {
            paused: false,
            owner_addr: factory_addr.clone(),
            croncat_factory_addr: factory_addr,
            chain_name: "atom".to_owned(),
            croncat_manager_key: ("manager".to_owned(), [0, 1]),
            croncat_agents_key: ("agents".to_owned(), [0, 1]),
            slot_granularity_time: SLOT_GRANULARITY_TIME,
            gas_base_fee: GAS_BASE_FEE,
            gas_action_fee: GAS_ACTION_FEE,
            gas_query_fee: GAS_QUERY_FEE,
            gas_limit: GAS_LIMIT,
        };

        assert_eq!(config, expected_config);
        // let manager_addr = init_manager(&mut app, &factory_addr);
    }

    #[test]
    fn custom_init() {
        let mut app = default_app();
        let factory_addr = init_factory(&mut app);

        let instantiate_msg: InstantiateMsg = InstantiateMsg {
            chain_name: "cron".to_owned(),
            owner_addr: Some(ANYONE.to_owned()),
            croncat_manager_key: ("definitely_not_manager".to_owned(), [4, 2]),
            croncat_agents_key: ("definitely_not_agents".to_owned(), [42, 0]),
            slot_granularity_time: Some(10),
            gas_base_fee: Some(1),
            gas_action_fee: Some(2),
            gas_query_fee: Some(3),
            gas_limit: Some(10),
        };
        let tasks_addr = init_tasks(&mut app, &instantiate_msg, &factory_addr);
        let config: Config = app
            .wrap()
            .query_wasm_smart(tasks_addr, &QueryMsg::Config {})
            .unwrap();

        let expected_config = Config {
            paused: false,
            owner_addr: Addr::unchecked(ANYONE),
            croncat_factory_addr: factory_addr,
            chain_name: "cron".to_owned(),
            croncat_manager_key: ("definitely_not_manager".to_owned(), [4, 2]),
            croncat_agents_key: ("definitely_not_agents".to_owned(), [42, 0]),
            slot_granularity_time: 10,
            gas_base_fee: 1,
            gas_action_fee: 2,
            gas_query_fee: 3,
            gas_limit: 10,
        };
        assert_eq!(config, expected_config);
    }

    #[test]
    fn failed_inits() {
        let mut app = default_app();
        let code_id = app.store_code(contracts::croncat_tasks_contract());

        let instantiate_msg: InstantiateMsg = InstantiateMsg {
            owner_addr: Some("InVA$LID_ADDR".to_owned()),
            ..default_instantiate_msg()
        };
        let contract_err: ContractError = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(ADMIN),
                &instantiate_msg,
                &[],
                "tasks",
                None,
            )
            .unwrap_err()
            .downcast()
            .unwrap();

        assert_eq!(
            contract_err,
            ContractError::Std(StdError::generic_err(
                "Invalid input: address not normalized"
            ))
        );
    }
}
