use crate::error::ContractError;
use crate::msg::*;
use cw_multi_test::{App, AppResponse, Executor};

use crate::tests::common::*;
use cosmwasm_std::{Addr, Coin, Uint128};


#[test]
fn test_contract_initialize_is_successfull() {
    let mut app = default_app();
    let contract_code_id = app.store_code(agent_contract());
    let (_, croncat_manager_addr) =
        init_croncat_manager_contract(&mut app, Some(ADMIN), Some(ADMIN.to_string()), Some(&[]));
    let init_msg = InstantiateMsg {
        owner_addr: Some(ADMIN.to_string()),
        agent_nomination_duration: None,
        min_tasks_per_agent: None,
        manager_addr: croncat_manager_addr.to_string(),
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
        agent_nomination_duration: None,
        min_tasks_per_agent: None,
        manager_addr: croncat_manager_addr.to_string(),
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
        manager_addr: String::new(),
        owner_addr: Some(ADMIN.to_string()),
        agent_nomination_duration: None,
        min_tasks_per_agent: None,
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

    assert_eq!(
        error,
        ContractError::InvalidCroncatManagerAddress {
            addr: String::new()
        }
    );
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

    let (_, croncat_manager_addr) =
        init_croncat_manager_contract(&mut app, Some(ADMIN), Some(ADMIN.to_string()), Some(&[]));

    //Check contract is paused and failing
    let mut config = mock_update_config(croncat_manager_addr.to_string().as_str());
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

#[test]
fn test_update_agent_is_successfull() {
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

    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::UpdateAgent {
            payable_account_id: ADMIN.to_string(),
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

    assert_eq!(
        agent_response.payable_account_id.to_string(),
        ADMIN.to_string()
    );
}

//UpdateAgent tests
#[test]
fn test_update_agent_fails() {
    let mut app = default_app();
    let (_, contract_addr) = init_agents_contract(&mut app, None, None, None, None);

    //Check contract fails when agent does not exist
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

    let error: ContractError = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &ExecuteMsg::UpdateAgent {
                payable_account_id: ADMIN.to_string(),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(error, ContractError::AgentNotRegistered);
    let (_, croncat_manager_addr) =
        init_croncat_manager_contract(&mut app, Some(ADMIN), Some(ADMIN.to_string()), Some(&[]));
    //Check contract is paused and failing
    let mut config = mock_update_config(croncat_manager_addr.to_string().as_str());
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
            &ExecuteMsg::UpdateAgent {
                payable_account_id: ADMIN.to_string(),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(error, ContractError::ContractPaused);
}

//UpdateAgent tests
#[test]
fn test_agent_check_in_successfull() {
    let mut app = default_app();
    let (_, contract_addr) = init_agents_contract(&mut app, None, None, None, None);

    register_agent(&mut app, &contract_addr, ANYONE, PARTICIPANT0).unwrap();
    register_agent(&mut app, &contract_addr, ADMIN, PARTICIPANT0).unwrap();
    app.update_block(|block| add_seconds_to_block(block, 500));
    on_task_created(&mut app, &contract_addr, ADMIN, "task1", 4);

    check_in_agent(&mut app, &contract_addr, ADMIN).unwrap();

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
}
#[test]
fn accept_nomination_agent() {
    let mut app = default_app();
    let (_, contract_addr) = init_agents_contract(&mut app, None, None, None, None);
    let mut total_tasks = 0;

    // Register AGENT1, who immediately becomes active
    register_agent(&mut app, &contract_addr, AGENT1, &AGENT_BENEFICIARY).unwrap();

    on_task_created(&mut app, &contract_addr, PARTICIPANT0, "task0", total_tasks);
    total_tasks += 1;

    assert_eq!(total_tasks, 1);

    // Register two agents
    register_agent(&mut app, &contract_addr, AGENT2, &AGENT_BENEFICIARY).unwrap();
    register_agent(&mut app, &contract_addr, AGENT3, &AGENT_BENEFICIARY).unwrap();

    let (agent_ids_res, num_active_agents, _) = get_agent_ids(&app, &contract_addr);
    assert_eq!(1, num_active_agents);
    assert_eq!(2, agent_ids_res.pending.len());

    on_task_created(&mut app, &contract_addr, PARTICIPANT1, "task1", total_tasks);
    total_tasks += 1;
    on_task_created(&mut app, &contract_addr, PARTICIPANT2, "task2", total_tasks);
    total_tasks += 1;
    on_task_created(&mut app, &contract_addr, PARTICIPANT3, "task3", total_tasks);
    total_tasks += 1;

    assert_eq!(total_tasks, 4);

    // Fast forward time a little
    app.update_block(|block| add_seconds_to_block(block, 19));
    app.update_block(|block| increment_block_height(block, None));

    let mut agent_status = get_agent_status(&mut app, &contract_addr, AGENT3, total_tasks)
        .unwrap()
        .unwrap()
        .status;
    assert_eq!(AgentStatus::Pending, agent_status);
    agent_status = get_agent_status(&mut app, &contract_addr, AGENT2, total_tasks)
        .unwrap()
        .unwrap()
        .status;
    assert_eq!(AgentStatus::Nominated, agent_status);

    // Attempt to accept nomination
    // First try with the agent second in line in the pending queue.
    // This should fail because it's not time for them yet.
    let mut check_in_res = check_in_agent(&mut app, &contract_addr, AGENT3);
    assert!(
        &check_in_res.is_err(),
        "Should throw error when agent in second position tries to nominate before their time."
    );
    assert_eq!(
        ContractError::TryLaterForNomination,
        check_in_res.unwrap_err().downcast().unwrap()
    );

    // Now try from person at the beginning of the pending queue
    // This agent should succeed
    check_in_res = check_in_agent(&mut app, &contract_addr, AGENT2);
    assert!(
        check_in_res.is_ok(),
        "Agent at the front of the pending queue should be allowed to nominate themselves"
    );

    // Check that active and pending queues are correct
    let (agent_ids_res, num_active_agents, _) = get_agent_ids(&app, &contract_addr);
    assert_eq!(2, num_active_agents);
    assert_eq!(1, agent_ids_res.pending.len());

    // The agent that was second in the queue is now first,
    // tries again, but there aren't enough tasks
    check_in_res = check_in_agent(&mut app, &contract_addr, AGENT3);

    let error_msg = check_in_res.unwrap_err();
    assert_eq!(
        ContractError::NotAcceptingNewAgents,
        error_msg.downcast().unwrap()
    );

    agent_status = get_agent_status(&mut app, &contract_addr, AGENT3, total_tasks)
        .unwrap()
        .unwrap()
        .status;
    assert_eq!(AgentStatus::Pending, agent_status);

    println!("start");
    on_task_created(&mut app, &contract_addr, PARTICIPANT3, "task4", total_tasks);
    total_tasks += 1;
    on_task_created(&mut app, &contract_addr, PARTICIPANT3, "task5", total_tasks);
    total_tasks += 1;
    on_task_created(&mut app, &contract_addr, PARTICIPANT3, "task6", total_tasks);
    total_tasks += 1;

    // Add another agent, since there's now the need
    register_agent(&mut app, &contract_addr, AGENT4, &AGENT_BENEFICIARY).unwrap();
    // Fast forward time past the duration of the first pending agent,
    // allowing the second to nominate themselves
    app.update_block(|block| add_seconds_to_block(block, 420));

    // Now that enough time has passed, both agents should see they're nominated
    agent_status = get_agent_status(&mut app, &contract_addr, AGENT3, total_tasks)
        .unwrap()
        .unwrap()
        .status;
    assert_eq!(AgentStatus::Nominated, agent_status);
    agent_status = get_agent_status(&mut app, &contract_addr, AGENT4, total_tasks)
        .unwrap()
        .unwrap()
        .status;
    assert_eq!(AgentStatus::Nominated, agent_status);

    // Agent second in line nominates themself
    check_in_res = check_in_agent(&mut app, &contract_addr, AGENT4);
    assert!(
        check_in_res.is_ok(),
        "Agent second in line should be able to nominate themselves"
    );

    let (_, _, num_pending_agents) = get_agent_ids(&app, &contract_addr);

    // Ensure the pending list is empty, having the earlier index booted
    assert_eq!(
        num_pending_agents, 0,
        "Expect the pending queue to be empty"
    );
}

#[test]
fn test_get_agent_status() {
    let mut app = default_app();
    let (_, contract_addr) = init_agents_contract(&mut app, None, None, None, None);
    let mut total_tasks = 0;

    let agent_status_res = get_agent_status(&mut app, &contract_addr, AGENT1, 0).unwrap();
    assert_eq!(None, agent_status_res);

    // Register AGENT1, who immediately becomes active
    let register_agent_res = register_agent(&mut app, &contract_addr, AGENT0, &AGENT_BENEFICIARY);
    // First registered agent becomes active
    assert!(
        register_agent_res.is_ok(),
        "Registering agent should succeed"
    );

    let agent_status_res = get_agent_status(&mut app, &contract_addr, AGENT0, 0);
    assert_eq!(
        AgentStatus::Active,
        agent_status_res.unwrap().unwrap().status
    );

    // Register an agent and make sure the status comes back as pending
    let register_agent_res = register_agent(&mut app, &contract_addr, AGENT1, PARTICIPANT1);
    assert!(
        register_agent_res.is_ok(),
        "Registering agent should succeed"
    );
    let agent_status_res = get_agent_status(&mut app, &contract_addr, AGENT1, total_tasks);
    assert_eq!(
        AgentStatus::Pending,
        agent_status_res.unwrap().unwrap().status,
        "New agent should be pending"
    );
    total_tasks += 3;
    on_task_created(&mut app, &contract_addr, PARTICIPANT2, "task2", total_tasks);

    // Agent status is nominated
    let agent_status_res = get_agent_status(&mut app, &contract_addr, AGENT1, total_tasks);

    assert_eq!(
        AgentStatus::Nominated,
        agent_status_res.unwrap().unwrap().status,
        "New agent should have nominated status"
    );
}

fn register_agent(
    app: &mut App,
    contract_addr: &Addr,
    agent: &str,
    beneficiary: &str,
) -> Result<AppResponse, anyhow::Error> {
    app.execute_contract(
        Addr::unchecked(agent),
        contract_addr.clone(),
        &ExecuteMsg::RegisterAgent {
            payable_account_id: Some(beneficiary.to_string()),
            cost: 1,
        },
        &[],
    )
}
fn get_agent_ids(app: &App, contract_addr: &Addr) -> (GetAgentIdsResponse, usize, usize) {
    let res: GetAgentIdsResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::GetAgentIds {
                skip: None,
                take: None,
            },
        )
        .unwrap();
    (res.clone(), res.active.len(), res.pending.len())
}

fn get_agent_status(
    app: &mut App,
    contract_addr: &Addr,
    agent: &str,
    total_tasks: u64,
) -> Result<Option<AgentResponse>, anyhow::Error> {
    let agent_info: Option<AgentResponse> = app.wrap().query_wasm_smart(
        &contract_addr.clone(),
        &QueryMsg::GetAgent {
            account_id: agent.to_string(),
            total_tasks,
        },
    )?;

    return Ok(agent_info);
}

fn check_in_agent(
    app: &mut App,
    contract_addr: &Addr,
    agent: &str,
) -> Result<AppResponse, anyhow::Error> {
    app.execute_contract(
        Addr::unchecked(agent),
        contract_addr.clone(),
        &ExecuteMsg::CheckInAgent {},
        &[],
    )
}

fn on_task_created(
    app: &mut App,
    contract_addr: &Addr,
    agent: &str,
    task_hash: &str,
    total_tasks: u64,
) -> AppResponse {
    app.execute_contract(
        Addr::unchecked(agent),
        contract_addr.clone(),
        &ExecuteMsg::OnTaskCreated {
            task_hash: task_hash.to_string(),
            total_tasks,
        },
        &[],
    )
    .expect("Error sending task created event")
}
