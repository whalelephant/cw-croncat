use crate::error::ContractError;
use crate::state::Config;
use crate::tests::helpers::{add_little_time, proper_instantiate};
use crate::CwCroncat;
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    coin, coins, from_slice, Addr, BlockInfo, CosmosMsg, DepsMut, MessageInfo, Response,
    StakingMsg, StdResult, Storage,
};
use cw_croncat_core::msg::{
    AgentTaskResponse, ExecuteMsg, GetAgentIdsResponse, InstantiateMsg, QueryMsg, TaskRequest,
    TaskResponse,
};
use cw_croncat_core::types::{Action, Agent, AgentResponse, AgentStatus, GenericBalance, Interval};
use cw_multi_test::{App, AppResponse, BankSudo, Executor, SudoMsg};

use super::helpers::{
    contract_template, ADMIN, AGENT0, AGENT1, AGENT2, AGENT3, AGENT4, AGENT_BENEFICIARY,
    NATIVE_DENOM, PARTICIPANT0, PARTICIPANT1, PARTICIPANT2, PARTICIPANT3, PARTICIPANT4,
    PARTICIPANT5, PARTICIPANT6,
};

fn get_task_total(app: &App, contract_addr: &Addr) -> usize {
    let res: Vec<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::GetTasks {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    res.len()
}

fn add_task_exec(app: &mut App, contract_addr: &Addr, sender: &str) -> AppResponse {
    let validator = String::from("you");
    let amount = coin(3, NATIVE_DENOM);
    let stake = StakingMsg::Delegate { validator, amount };
    let msg: CosmosMsg = stake.clone().into();
    let send_funds = coins(500_000, NATIVE_DENOM);
    app.execute_contract(
        Addr::unchecked(sender),
        contract_addr.clone(),
        &ExecuteMsg::CreateTask {
            task: TaskRequest {
                interval: Interval::Immediate,
                boundary: None,
                stop_on_fail: false,
                actions: vec![Action {
                    msg,
                    gas_limit: Some(150_000),
                }],
                rules: None,
                cw20_coins: vec![],
            },
        },
        send_funds.as_ref(),
    )
    .expect("Error adding task")
}

fn add_block_task_exec(
    app: &mut App,
    contract_addr: &Addr,
    sender: &str,
    block_num: u64,
) -> AppResponse {
    let validator = String::from("you");
    let amount = coin(3, NATIVE_DENOM);
    let stake = StakingMsg::Delegate { validator, amount };
    let msg: CosmosMsg = stake.clone().into();
    let send_funds = coins(500_000, NATIVE_DENOM);
    app.execute_contract(
        Addr::unchecked(sender),
        contract_addr.clone(),
        &ExecuteMsg::CreateTask {
            task: TaskRequest {
                interval: Interval::Block(block_num),
                boundary: None,
                stop_on_fail: false,
                actions: vec![Action {
                    msg,
                    gas_limit: Some(150_000),
                }],
                rules: None,
                cw20_coins: vec![],
            },
        },
        send_funds.as_ref(),
    )
    .expect("Error adding task")
}

fn add_cron_task_exec(
    app: &mut App,
    contract_addr: &Addr,
    sender: &str,
    num_minutes: u64,
) -> AppResponse {
    let validator = String::from("you");
    let amount = coin(3, NATIVE_DENOM);
    let stake = StakingMsg::Delegate { validator, amount };
    let msg: CosmosMsg = stake.clone().into();
    let send_funds = coins(500_000, NATIVE_DENOM);
    app.execute_contract(
        Addr::unchecked(sender),
        contract_addr.clone(),
        &ExecuteMsg::CreateTask {
            task: TaskRequest {
                interval: Interval::Cron(format!("* {} * * * *", num_minutes)),
                boundary: None,
                stop_on_fail: false,
                actions: vec![Action {
                    msg,
                    gas_limit: Some(150_000),
                }],
                rules: None,
                cw20_coins: vec![],
            },
        },
        send_funds.as_ref(),
    )
    .expect("Error adding task")
}

fn contract_create_task(
    contract: &CwCroncat,
    deps: DepsMut,
    info: &MessageInfo,
) -> Result<Response, ContractError> {
    // try adding task without app
    let validator = String::from("you");
    let amount = coin(3, NATIVE_DENOM);
    let stake = StakingMsg::Delegate { validator, amount };
    let msg: CosmosMsg = stake.clone().into();
    // let send_funds = coins(1, NATIVE_DENOM);

    contract.create_task(
        deps,
        info.clone(),
        mock_env(),
        TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: msg.clone(),
                gas_limit: Some(150_000),
            }],
            rules: None,
            cw20_coins: vec![],
        },
    )
}

fn contract_register_agent(
    sender: &str,
    contract: &mut CwCroncat,
    deps: DepsMut,
) -> Result<Response, ContractError> {
    contract.execute(
        deps,
        mock_env(),
        MessageInfo {
            sender: Addr::unchecked(sender),
            funds: vec![],
        },
        ExecuteMsg::RegisterAgent {
            payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
        },
    )
}

fn get_stored_agent_status(app: &mut App, contract_addr: &Addr, agent: &str) -> AgentStatus {
    let agent_info: AgentResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetAgent {
                account_id: agent.to_string(),
            },
        )
        .expect("Error getting agent status");
    agent_info.status
}

fn register_agent_exec(
    app: &mut App,
    contract_addr: &Addr,
    agent: &str,
    beneficiary: &str,
) -> AppResponse {
    app.execute_contract(
        Addr::unchecked(agent),
        contract_addr.clone(),
        &ExecuteMsg::RegisterAgent {
            payable_account_id: Some(beneficiary.to_string()),
        },
        &[],
    )
    .expect("Error registering agent")
}

fn check_in_exec(
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

fn get_agent_ids(app: &App, contract_addr: &Addr) -> (GetAgentIdsResponse, usize, usize) {
    let res: GetAgentIdsResponse = app
        .wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::GetAgentIds {})
        .unwrap();
    (res.clone(), res.active.len(), res.pending.len())
}

pub fn add_one_duration_of_time(block: &mut BlockInfo) {
    // block.time = block.time.plus_seconds(360);
    block.time = block.time.plus_seconds(420);
    block.height += 1;
}

#[test]
fn test_instantiate_sets_balance() {
    let mut app = App::default();
    let croncat_code = app.store_code(contract_template());

    let sent_funds = coins(50, "ugrape");

    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: "grapestem".to_string(),
        amount: sent_funds.clone(),
    }))
    .unwrap();

    let croncat = app
        .instantiate_contract(
            croncat_code,
            Addr::unchecked("grapestem"),
            &InstantiateMsg {
                denom: "grape".to_string(),
                cw_rules_addr: "grapestem".to_string(),
                owner_id: None,
                gas_base_fee: None,
                agent_nomination_duration: None,
            },
            &sent_funds,
            "cw croncat",
            None,
        )
        .unwrap();

    let config: Config = from_slice(
        &app.wrap()
            .query_wasm_raw(&croncat, b"config")
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(config.available_balance.native, sent_funds)
}

#[test]
fn register_agent_fail_cases() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    // start first register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };

    // Test funds fail register if sent
    let rereg_err = app
        .execute_contract(
            Addr::unchecked(AGENT1),
            contract_addr.clone(),
            &msg,
            &coins(37, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::CustomError {
            val: "Do not attach funds".to_string()
        },
        rereg_err.downcast().unwrap()
    );

    // Test Can't register if contract is paused
    let payload_1 = ExecuteMsg::UpdateSettings {
        paused: Some(true),
        owner_id: None,
        // treasury_id: None,
        agent_fee: None,
        min_tasks_per_agent: None,
        agents_eject_threshold: None,
        gas_for_one_native: None,
        proxy_callback_gas: None,
        slot_granularity: None,
    };

    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &payload_1,
        &[],
    )
    .unwrap();
    let rereg_err = app
        .execute_contract(Addr::unchecked(AGENT1), contract_addr.clone(), &msg, &[])
        .unwrap_err();
    assert_eq!(
        ContractError::ContractPaused {
            val: "Register agent paused".to_string()
        },
        rereg_err.downcast().unwrap()
    );

    // Test wallet rejected if doesnt have enough funds
    let payload_2 = ExecuteMsg::UpdateSettings {
        paused: Some(false),
        owner_id: None,
        // treasury_id: None,
        agent_fee: None,
        min_tasks_per_agent: None,
        agents_eject_threshold: None,
        gas_for_one_native: Some(1),
        proxy_callback_gas: None,
        slot_granularity: None,
    };

    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &payload_2,
        &[],
    )
    .unwrap();
    app.send_tokens(
        Addr::unchecked(AGENT0),
        Addr::unchecked(AGENT1),
        &coins(1_900_000, "atom"),
    )
    .unwrap();
    let rereg_err = app
        .execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap_err();
    assert_eq!(
        ContractError::CustomError {
            val: "Insufficient funds".to_string()
        },
        rereg_err.downcast().unwrap()
    );
}

#[test]
fn register_agent() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();
    let blk_time = app.block_info().time;

    // start first register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT1), contract_addr.clone(), &msg, &[])
        .unwrap();

    // check state to see if worked
    let (_, num_active_agents, num_pending_agents) = get_agent_ids(&app, &contract_addr);
    assert_eq!(1, num_active_agents);
    assert_eq!(0, num_pending_agents);

    // message response matches expectations (same block, all the defaults)
    let agent_info: AgentResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetAgent {
                account_id: AGENT1.to_string(),
            },
        )
        .unwrap();
    println!("agent_infoagent_info {:?}", agent_info);
    assert_eq!(AgentStatus::Active, agent_info.status);
    assert_eq!(
        Addr::unchecked(AGENT_BENEFICIARY),
        agent_info.payable_account_id
    );
    assert_eq!(GenericBalance::default(), agent_info.balance);
    assert_eq!(0, agent_info.total_tasks_executed);
    assert_eq!(0, agent_info.last_missed_slot);
    assert_eq!(blk_time, agent_info.register_start);

    // test fail if try to re-register
    let rereg_err = app
        .execute_contract(Addr::unchecked(AGENT1), contract_addr.clone(), &msg, &[])
        .unwrap_err();
    assert_eq!(
        ContractError::CustomError {
            val: "Agent already exists".to_string()
        },
        rereg_err.downcast().unwrap()
    );

    // test another register, put into pending queue
    let msg2 = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT2), contract_addr.clone(), &msg2, &[])
        .unwrap();

    // check state to see if worked

    let (_, num_active_agents, num_pending_agents) = get_agent_ids(&app, &contract_addr);
    assert_eq!(1, num_active_agents);
    assert_eq!(1, num_pending_agents);
}

#[test]
fn update_agent() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    // start first register
    let msg1 = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT1), contract_addr.clone(), &msg1, &[])
        .unwrap();

    // Fails for non-existent agents
    let msg = ExecuteMsg::UpdateAgent {
        payable_account_id: AGENT0.to_string(),
    };
    let update_err = app
        .execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap_err();
    assert_eq!(
        ContractError::AgentNotRegistered {},
        update_err.downcast().unwrap()
    );

    app.execute_contract(Addr::unchecked(AGENT1), contract_addr.clone(), &msg, &[])
        .unwrap();

    // payable account was in fact updated
    let agent_info: Agent = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetAgent {
                account_id: AGENT1.to_string(),
            },
        )
        .unwrap();
    assert_eq!(Addr::unchecked(AGENT0), agent_info.payable_account_id);
}

#[test]
fn unregister_agent() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    // start first register
    let msg1 = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT1), contract_addr.clone(), &msg1, &[])
        .unwrap();

    // Fails for non-exist agents
    let unreg_msg = ExecuteMsg::UnregisterAgent {};
    let update_err = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &unreg_msg,
            &[],
        )
        .unwrap_err();
    assert_eq!(
        ContractError::AgentNotRegistered {},
        update_err.downcast().unwrap()
    );

    // Get quick data about account before, to compare later
    let agent_bal = app
        .wrap()
        .query_balance(&Addr::unchecked(AGENT1), NATIVE_DENOM)
        .unwrap();
    assert_eq!(agent_bal, coin(2_000_000, NATIVE_DENOM));

    // Attempt the unregister
    app.execute_contract(
        Addr::unchecked(AGENT1),
        contract_addr.clone(),
        &unreg_msg,
        &[],
    )
    .unwrap();

    // Agent should not exist now
    let update_err = app
        .execute_contract(
            Addr::unchecked(AGENT1),
            contract_addr.clone(),
            &unreg_msg,
            &[],
        )
        .unwrap_err();
    assert_eq!(
        ContractError::AgentNotRegistered {},
        update_err.downcast().unwrap()
    );

    // Check that the agent was removed from the list of active or pending agents
    let (_, num_active_agents, num_pending_agents) = get_agent_ids(&app, &contract_addr);
    assert_eq!(0, num_active_agents);
    assert_eq!(0, num_pending_agents);

    // Agent should have appropriate balance change
    // NOTE: Needs further checks when tasks can be performed
    let agent_bal = app
        .wrap()
        .query_balance(&Addr::unchecked(AGENT1), NATIVE_DENOM)
        .unwrap();
    assert_eq!(agent_bal, coin(2000000, NATIVE_DENOM));
}

#[test]
fn withdraw_agent_balance() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    // start first register
    let msg1 = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT1), contract_addr.clone(), &msg1, &[])
        .unwrap();

    // Fails for non-existent agents
    let wthdrw_msg = ExecuteMsg::WithdrawReward {};
    let update_err = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &wthdrw_msg,
            &[],
        )
        .unwrap_err();
    assert_eq!(
        ContractError::AgentNotRegistered {},
        update_err.downcast().unwrap()
    );

    // Get quick data about account before, to compare later
    let agent_bal = app
        .wrap()
        .query_balance(&Addr::unchecked(AGENT1), NATIVE_DENOM)
        .unwrap();
    assert_eq!(agent_bal, coin(2_000_000, NATIVE_DENOM));

    // Attempt the withdraw
    app.execute_contract(
        Addr::unchecked(AGENT1),
        contract_addr.clone(),
        &wthdrw_msg,
        &[],
    )
    .unwrap();

    // Agent should have appropriate balance change
    // NOTE: Needs further checks when tasks can be performed
    let agent_bal = app
        .wrap()
        .query_balance(&Addr::unchecked(AGENT1), NATIVE_DENOM)
        .unwrap();
    assert_eq!(agent_bal, coin(2_000_000, NATIVE_DENOM));
}

#[test]
fn accept_nomination_agent() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    // Register AGENT1, who immediately becomes active
    register_agent_exec(&mut app, &contract_addr, AGENT1, &AGENT_BENEFICIARY);
    let res = add_task_exec(&mut app, &contract_addr, PARTICIPANT0);
    let task_hash = res.events[1].attributes[4].clone().value;
    assert_eq!(
        "7ea9a6d5ef5c78cb168afa96b43b5843b8f880627aa0580f4311403f907cbf93", task_hash,
        "Unexpected task hash"
    );

    let msg_query_task = QueryMsg::GetTask { task_hash };
    let query_task_res: StdResult<Option<TaskResponse>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg_query_task);
    assert!(
        query_task_res.is_ok(),
        "Did not successfully find the newly added task"
    );

    let mut num_tasks = get_task_total(&app, &contract_addr);
    assert_eq!(num_tasks, 1);

    // Now the task ratio is 1:2 (one agent per two tasks)
    // No agent should be allowed to join or accept nomination
    // Check that this fails

    // Register two agents
    register_agent_exec(&mut app, &contract_addr, AGENT2, &AGENT_BENEFICIARY);
    // Later, we'll have this agent try to nominate themselves before their time
    register_agent_exec(&mut app, &contract_addr, AGENT3, &AGENT_BENEFICIARY);

    let (agent_ids_res, num_active_agents, _) = get_agent_ids(&app, &contract_addr);
    assert_eq!(1, num_active_agents);
    assert_eq!(2, agent_ids_res.pending.len());

    // Add three more tasks, so we can nominate another agent
    add_task_exec(&mut app, &contract_addr, PARTICIPANT1);
    add_task_exec(&mut app, &contract_addr, PARTICIPANT2);
    add_task_exec(&mut app, &contract_addr, PARTICIPANT3);

    num_tasks = get_task_total(&app, &contract_addr);
    assert_eq!(num_tasks, 4);

    // Fast forward time a little
    app.update_block(add_little_time);

    let mut agent_status = get_stored_agent_status(&mut app, &contract_addr, AGENT3);
    assert_eq!(AgentStatus::Pending, agent_status);
    agent_status = get_stored_agent_status(&mut app, &contract_addr, AGENT2);
    assert_eq!(AgentStatus::Nominated, agent_status);

    // Attempt to accept nomination
    // First try with the agent second in line in the pending queue.
    // This should fail because it's not time for them yet.
    let mut check_in_res = check_in_exec(&mut app, &contract_addr, AGENT3);
    assert!(
        &check_in_res.is_err(),
        "Should throw error when agent in second position tries to nominate before their time."
    );
    assert_eq!(
        ContractError::CustomError {
            val: "Must wait longer before accepting nomination".to_string()
        },
        check_in_res.unwrap_err().downcast().unwrap()
    );

    // Now try from person at the beginning of the pending queue
    // This agent should succeed
    check_in_res = check_in_exec(&mut app, &contract_addr, AGENT2);
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
    check_in_res = check_in_exec(&mut app, &contract_addr, AGENT3);

    let error_msg = check_in_res.unwrap_err();
    assert_eq!(
        ContractError::CustomError {
            val: "Not accepting new agents".to_string()
        },
        error_msg.downcast().unwrap()
    );

    agent_status = get_stored_agent_status(&mut app, &contract_addr, AGENT3);
    assert_eq!(AgentStatus::Pending, agent_status);

    // Again, add three more tasks so we can nominate another agent
    add_task_exec(&mut app, &contract_addr, PARTICIPANT4);
    add_task_exec(&mut app, &contract_addr, PARTICIPANT5);
    add_task_exec(&mut app, &contract_addr, PARTICIPANT6);

    num_tasks = get_task_total(&app, &contract_addr);
    assert_eq!(num_tasks, 7);

    // Add another agent, since there's now the need
    register_agent_exec(&mut app, &contract_addr, AGENT4, &AGENT_BENEFICIARY);
    // Fast forward time past the duration of the first pending agent,
    // allowing the second to nominate themselves
    app.update_block(add_one_duration_of_time);

    // Now that enough time has passed, both agents should see they're nominated
    agent_status = get_stored_agent_status(&mut app, &contract_addr, AGENT3);
    assert_eq!(AgentStatus::Nominated, agent_status);
    agent_status = get_stored_agent_status(&mut app, &contract_addr, AGENT4);
    assert_eq!(AgentStatus::Nominated, agent_status);

    // Agent second in line nominates themself
    check_in_res = check_in_exec(&mut app, &contract_addr, AGENT4);
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
    // Give the contract and the agents balances
    let mut deps = cosmwasm_std::testing::mock_dependencies_with_balances(&[
        (&MOCK_CONTRACT_ADDR, &[coin(6000, NATIVE_DENOM)]),
        (&AGENT0, &[coin(2_000_000, NATIVE_DENOM)]),
        (&AGENT1, &[coin(2_000_000, NATIVE_DENOM)]),
    ]);
    let mut contract = CwCroncat::default();

    // Instantiate
    let msg = InstantiateMsg {
        denom: NATIVE_DENOM.to_string(),
        owner_id: None,
        gas_base_fee: None,
        agent_nomination_duration: Some(360),
        cw_rules_addr: "todo".to_string(),
    };
    let mut info = mock_info(AGENT0, &coins(900_000, NATIVE_DENOM));
    let res_init = contract
        .instantiate(deps.as_mut(), mock_env(), info.clone(), msg)
        .unwrap();
    assert_eq!(0, res_init.messages.len());

    let mut agent_status_res =
        contract.get_agent_status(&deps.storage, mock_env(), Addr::unchecked(AGENT0));
    assert_eq!(Err(ContractError::AgentNotRegistered {}), agent_status_res);

    let agent_active_queue_opt: Vec<Addr> = match deps.storage.get("agent_active_queue".as_bytes())
    {
        Some(vec) => from_slice(vec.as_ref()).expect("Could not load agent active queue"),
        None => {
            panic!("Uninitialized agent_active_queue_opt");
        }
    };
    assert!(
        agent_active_queue_opt.is_empty(),
        "Should not have an active queue yet"
    );

    // First registered agent becomes active
    let mut register_agent_res = contract_register_agent(AGENT0, &mut contract, deps.as_mut());
    assert!(
        register_agent_res.is_ok(),
        "Registering agent should succeed"
    );

    agent_status_res =
        contract.get_agent_status(&deps.storage, mock_env(), Addr::unchecked(AGENT0));
    assert_eq!(AgentStatus::Active, agent_status_res.unwrap());

    // Add two tasks
    let mut res_add_task = contract_create_task(&contract, deps.as_mut(), &info);
    assert!(res_add_task.is_ok(), "Adding task should succeed.");
    // Change sender so it's not a duplicate task
    info.sender = Addr::unchecked(PARTICIPANT0);
    res_add_task = contract_create_task(&contract, deps.as_mut(), &info);
    assert!(res_add_task.is_ok(), "Adding task should succeed.");

    // Register an agent and make sure the status comes back as pending
    register_agent_res = contract_register_agent(AGENT1, &mut contract, deps.as_mut());
    assert!(
        register_agent_res.is_ok(),
        "Registering agent should succeed"
    );
    agent_status_res =
        contract.get_agent_status(&deps.storage, mock_env(), Addr::unchecked(AGENT1));
    assert_eq!(
        AgentStatus::Pending,
        agent_status_res.unwrap(),
        "New agent should be pending"
    );

    // Two more tasks are added
    info.sender = Addr::unchecked(PARTICIPANT1);
    res_add_task = contract_create_task(&contract, deps.as_mut(), &info);
    assert!(res_add_task.is_ok(), "Adding task should succeed.");
    info.sender = Addr::unchecked(PARTICIPANT2);
    res_add_task = contract_create_task(&contract, deps.as_mut(), &info);
    assert!(res_add_task.is_ok(), "Adding task should succeed.");

    // Agent status is nominated
    agent_status_res =
        contract.get_agent_status(&deps.storage, mock_env(), Addr::unchecked(AGENT1));
    assert_eq!(
        AgentStatus::Nominated,
        agent_status_res.unwrap(),
        "New agent should have nominated status"
    );
}

#[test]
fn test_query_get_agent_tasks() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();
    let block_info = app.block_info();
    println!(
        "test aloha\n\tcurrent block: {}\n\tcurrent time: {}",
        block_info.height,
        block_info.time.nanos()
    );

    // Register AGENT1, who immediately becomes active
    register_agent_exec(&mut app, &contract_addr, AGENT1, &AGENT_BENEFICIARY);
    // Add five tasks total
    // Three of them are block-based
    add_block_task_exec(
        &mut app,
        &contract_addr,
        PARTICIPANT0,
        block_info.height + 6,
    );
    add_block_task_exec(
        &mut app,
        &contract_addr,
        PARTICIPANT1,
        block_info.height + 66,
    );
    add_block_task_exec(
        &mut app,
        &contract_addr,
        PARTICIPANT2,
        block_info.height + 67,
    );
    // add_block_task_exec(&mut app, &contract_addr, PARTICIPANT3, block_info.height + 131);
    // Two tasks use Cron instead of Block (for task interval)
    add_cron_task_exec(&mut app, &contract_addr, PARTICIPANT4, 6); // 3 minutes
    add_cron_task_exec(&mut app, &contract_addr, PARTICIPANT5, 53); // 53 minutes
    let num_tasks = get_task_total(&app, &contract_addr);
    assert_eq!(num_tasks, 5);

    // Now the task ratio is 1:2 (one agent per two tasks)
    // Register two agents, the first one succeeding
    register_agent_exec(&mut app, &contract_addr, AGENT2, &AGENT_BENEFICIARY);
    assert!(check_in_exec(&mut app, &contract_addr, AGENT2).is_ok());
    // This next agent should fail because there's no enough tasks yet
    // Later, we'll have this agent try to nominate themselves before their time
    register_agent_exec(&mut app, &contract_addr, AGENT3, &AGENT_BENEFICIARY);
    let failed_check_in = check_in_exec(&mut app, &contract_addr, AGENT3);
    assert_eq!(
        ContractError::CustomError {
            val: "Not accepting new agents".to_string()
        },
        failed_check_in.unwrap_err().downcast().unwrap()
    );

    let (_, num_active_agents, num_pending_agents) = get_agent_ids(&app, &contract_addr);
    assert_eq!(2, num_active_agents);
    assert_eq!(1, num_pending_agents);

    // Fast forward time a little
    app.update_block(|block| {
        let height = 666;
        block.time = block.time.plus_seconds(6 * height); // ~6 sec block time
        block.height = block.height + height;
    });

    // What happens when the only active agent queries to see if there's work for them
    // calls:
    // fn query_get_agent_tasks
    let mut msg_agent_tasks = QueryMsg::GetAgentTasks {
        account_id: AGENT1.to_string(),
    };
    let mut query_task_res: StdResult<Option<AgentTaskResponse>> = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg_agent_tasks);
    println!(
        "test aloha query_task_res0 {:#?}",
        query_task_res.as_ref().unwrap()
    );
    assert!(
        query_task_res.is_ok(),
        "Did not successfully find the newly added task"
    );
    msg_agent_tasks = QueryMsg::GetAgentTasks {
        account_id: AGENT2.to_string(),
    };
    query_task_res = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg_agent_tasks);
    println!("test aloha query_task_res1 {:#?}", query_task_res.unwrap());
    // Should fail for random user not in the active queue
    msg_agent_tasks = QueryMsg::GetAgentTasks {
        // rando account
        account_id: "juno1kqfjv53g7ll9u6ngvsu5l5nfv9ht24m4q4gdqz".to_string(),
    };
    query_task_res = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &msg_agent_tasks);
    println!("aloha query_task_res {:?}", query_task_res);
}
