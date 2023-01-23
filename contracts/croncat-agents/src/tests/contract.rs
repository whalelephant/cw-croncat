use crate::error::ContractError;
use crate::msg::*;
use croncat_sdk_tasks::types::SlotType;
use cw_multi_test::Executor;

use crate::balancer::{Balancer, RoundRobinBalancer};
use crate::state::{AGENTS_ACTIVE, AGENT_STATS};
use crate::tests::common::{
    agent_contract, default_app, mock_update_config, ADMIN, AGENT0, AGENT1, AGENT2, AGENT3, AGENT4,
    AGENT5, ANYONE,
};
use cosmwasm_std::testing::{
    mock_dependencies_with_balance, mock_env, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{coins, Addr, Coin, Empty, Env, MemoryStorage, OwnedDeps, StdError, Uint128};

use super::common::{init_agents_contract, mock_config, NATIVE_DENOM};

#[test]
fn test_contract_initialize_is_successfull() {
    let mut app = default_app();
    let contract_code_id = app.store_code(agent_contract());

    let init_msg = InstantiateMsg {
        owner_addr: Some(ADMIN.to_string()),
        native_denom: Some(NATIVE_DENOM.to_string()),
        agent_nomination_duration: None,
    };
    let contract_addr = app
        .instantiate_contract(
            contract_code_id,
            Addr::unchecked(ADMIN),
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
            Addr::unchecked(ANYONE),
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
#[test]
fn test_contract_initialize_fail_cases() {
    let mut app = default_app();
    let contract_code_id = app.store_code(agent_contract());

    let init_msg = InstantiateMsg {
        owner_addr: Some(ADMIN.to_string()),
        native_denom: Some(NATIVE_DENOM.to_string()),
        agent_nomination_duration: None,
    };
    let error: ContractError = app
        .instantiate_contract(
            contract_code_id,
            Addr::unchecked(ANYONE),
            &init_msg,
            &[],
            "agents",
            None,
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(error, ContractError::InvalidNativeDenom { denom: None });
}
//RegisterAgent
#[test]
fn test_register_agent_is_successfull() {
    let mut app = default_app();
    let (_, contract_addr) = init_agents_contract(&mut app, None, None, None, None);
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::RegisterAgent {
            payable_account_id: Some(ANYONE.to_string()),
            cost: 1,
        },
        &[],
    )
    .unwrap();

    let agent_response: AgentResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::GetAgent {
                account_id: ADMIN.to_string(),
                total_tasks: 10,
            },
        )
        .unwrap();

    assert_eq!(agent_response.status, AgentStatus::Active);
    assert_eq!(agent_response.total_tasks_executed, 0);
    assert_eq!(agent_response.balance, Uint128::new(0));
}

#[test]
fn test_register_agent_fails() {
    let mut app = default_app();
    let (_, contract_addr) = init_agents_contract(&mut app, None, None, None, None);
    let error: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &ExecuteMsg::RegisterAgent {
                payable_account_id: Some(ANYONE.to_string()),
                cost: 1,
            },
            &[Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(10),
            }],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(error, ContractError::NoFundsShouldBeAttached);

    //Check contract is paused and failing
    let mut config = mock_update_config();
    config.paused = Some(true);
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::UpdateConfig(Box::new(config)),
        &[],
    )
    .unwrap();

    let error: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &ExecuteMsg::RegisterAgent {
                payable_account_id: Some(ANYONE.to_string()),
                cost: 1,
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(error, ContractError::ContractPaused);
}
