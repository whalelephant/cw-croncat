use crate::msg::*;
use croncat_sdk_tasks::types::SlotType;
use cw_multi_test::Executor;

use crate::balancer::{Balancer, RoundRobinBalancer};
use crate::state::{AGENTS_ACTIVE, AGENT_STATS};
use crate::tests::common::{
    agent_contract, default_app, mock_instantiate, ADMIN, AGENT0, AGENT1, AGENT2, AGENT3, AGENT4,
    AGENT5, ANYONE,
};
use cosmwasm_std::testing::{
    mock_dependencies_with_balance, mock_env, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{coins, Addr, Empty, Env, MemoryStorage, OwnedDeps};

use super::common::{mock_config, NATIVE_DENOM};

#[test]
fn test_contract_initialize_is_successfull() {
    let mut app = default_app();
    let contract_code_id = app.store_code(agent_contract());
    let admin_unchecked = Addr::unchecked(ADMIN);
    let anyone_unchecked = Addr::unchecked(ANYONE);

    let init_msg = InstantiateMsg {
        owner_addr: Some(ADMIN.to_string()),
        native_denom: Some(NATIVE_DENOM.to_string()),
        agent_nomination_duration: None,
    };
    let contract_addr = app
        .instantiate_contract(
            contract_code_id,
            admin_unchecked,
            &init_msg,
            &[],
            "agents",
            None,
        )
        .unwrap();

    let config: Config = app
        .wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::Config {})
        .unwrap();
    assert_eq!(config.owner_addr, Addr::unchecked(ADMIN));

    let init_msg = InstantiateMsg {
        owner_addr: Some(ANYONE.to_string()),
        native_denom: Some(NATIVE_DENOM.to_string()),
        agent_nomination_duration: None,
    };

    let contract_addr = app
        .instantiate_contract(
            contract_code_id,
            anyone_unchecked,
            &init_msg,
            &[],
            "agents",
            None,
        )
        .unwrap();

    let config: Config = app
        .wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::Config {})
        .unwrap();
    assert_eq!(config.owner_addr, Addr::unchecked(ANYONE));
}
