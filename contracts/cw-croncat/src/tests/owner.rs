use crate::error::ContractError;
use crate::state::CwCroncat;
use crate::tests::helpers::{
    add_little_time, proper_instantiate, ADMIN, AGENT0, AGENT_BENEFICIARY, ANYONE, NATIVE_DENOM,
};
use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
use cosmwasm_std::{
    coin, coins, from_binary, to_binary, Addr, CosmosMsg, MessageInfo, StdResult, Uint64, WasmMsg,
};
use cw20::Balance;
use cw_croncat_core::msg::{
    CwCroncatResponse, ExecuteMsg, GetBalancesResponse, GetConfigResponse, InstantiateMsg,
    QueryMsg, RoundRobinBalancerModeResponse, TaskRequest,
};
use cw_croncat_core::types::{Action, Boundary, Interval};
use cw_multi_test::Executor;

#[test]
fn update_settings() {
    let mut deps = mock_dependencies_with_balance(&coins(200, ""));
    let mut store = CwCroncat::default();

    let msg = InstantiateMsg {
        denom: NATIVE_DENOM.to_string(),
        cw_rules_addr: "todo".to_string(),
        owner_id: None,
        gas_base_fee: None,
        gas_action_fee: None,
        gas_fraction: None,
        agent_nomination_duration: Some(360),
    };
    let info = MessageInfo {
        sender: Addr::unchecked("creator"),
        funds: vec![],
    };
    mock_info("creator", &coins(0, "meow"));
    let res_init = store
        .instantiate(deps.as_mut(), mock_env(), info.clone(), msg)
        .unwrap();
    assert_eq!(0, res_init.messages.len());

    let payload = ExecuteMsg::UpdateSettings {
        paused: Some(true),
        owner_id: None,
        // treasury_id: None,
        agent_fee: None,
        min_tasks_per_agent: None,
        agents_eject_threshold: None,
        gas_fraction: None,
        proxy_callback_gas: None,
        slot_granularity_time: None,
        gas_base_fee: None,
        gas_action_fee: None,
    };

    // non-owner fails
    let unauth_info = MessageInfo {
        sender: Addr::unchecked("michael_scott"),
        funds: vec![],
    };
    let res_fail = store.execute(deps.as_mut(), mock_env(), unauth_info, payload.clone());
    match res_fail {
        Err(ContractError::Unauthorized {}) => {}
        _ => panic!("Must return unauthorized error"),
    }

    // non-zero deposit fails
    let with_deposit_info = mock_info("owner_id", &coins(1000, "meow"));
    let res_fail = store.execute(
        deps.as_mut(),
        mock_env(),
        with_deposit_info,
        payload.clone(),
    );
    match res_fail {
        Err(ContractError::AttachedDeposit {}) => {}
        _ => panic!("Must return deposit error"),
    }

    // do the right thing
    let res_exec = store
        .execute(deps.as_mut(), mock_env(), info.clone(), payload)
        .unwrap();
    assert_eq!(0, res_exec.messages.len());

    // it worked, let's query the state
    let res = store
        .query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {})
        .unwrap();
    let value: GetConfigResponse = from_binary(&res).unwrap();
    assert_eq!(true, value.paused);
    assert_eq!(info.sender, value.owner_id);
}

#[test]
fn move_balances_auth_checks() {
    let mut deps = mock_dependencies_with_balance(&coins(200000000, NATIVE_DENOM));
    let mut store = CwCroncat::default();
    let info = mock_info("owner_id", &coins(1000, "meow"));
    let unauth_info = mock_info("michael_scott", &coins(2, "shrute_bucks"));
    let exist_bal = vec![Balance::from(coins(2, NATIVE_DENOM))];
    let non_exist_bal = vec![Balance::from(coins(2, "shrute_bucks"))];

    // instantiate with owner, then add treasury
    let msg = InstantiateMsg {
        denom: NATIVE_DENOM.to_string(),
        owner_id: None,
        gas_action_fee: None,
        gas_fraction: None,
        agent_nomination_duration: Some(360),
        cw_rules_addr: "todo".to_string(),
        gas_base_fee: None,
    };
    let res_init = store
        .instantiate(deps.as_mut(), mock_env(), info.clone(), msg)
        .unwrap();
    assert!(res_init.messages.is_empty());

    let payload = ExecuteMsg::UpdateSettings {
        paused: None,
        owner_id: None,
        // treasury_id: Some(Addr::unchecked("money_bags")),
        agent_fee: None,
        min_tasks_per_agent: None,
        agents_eject_threshold: None,
        gas_fraction: None,
        proxy_callback_gas: None,
        slot_granularity_time: None,
        gas_base_fee: None,
        gas_action_fee: None,
    };
    let info_setting = mock_info("owner_id", &coins(0, "meow"));
    let res_exec = store
        .execute(deps.as_mut(), mock_env(), info_setting, payload)
        .unwrap();
    assert!(res_exec.messages.is_empty());

    // try to move funds as non-owner
    let msg_move_1 = ExecuteMsg::MoveBalances {
        balances: non_exist_bal,
        account_id: "scammer".to_string(),
    };
    let res_fail_1 = store.execute(deps.as_mut(), mock_env(), unauth_info, msg_move_1);
    match res_fail_1 {
        Err(ContractError::Unauthorized {}) => {}
        _ => panic!("Must return unauthorized error"),
    }

    // try to move funds to account other than treasury or owner
    let msg_move_2 = ExecuteMsg::MoveBalances {
        balances: exist_bal.clone(),
        account_id: "scammer".to_string(),
    };
    let res_fail_2 = store.execute(deps.as_mut(), mock_env(), info.clone(), msg_move_2);
    match res_fail_2 {
        Err(ContractError::CustomError { .. }) => {}
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn move_balances_native() {
    let mut deps = mock_dependencies_with_balance(&coins(200000000, NATIVE_DENOM));
    let mut store = CwCroncat::default();
    let info = mock_info(
        "owner_id",
        &vec![coin(200000000, NATIVE_DENOM), coin(1000, "meow")],
    );
    let exist_bal = vec![Balance::from(coins(2, NATIVE_DENOM))];
    let spensive_bal = vec![Balance::from(coins(2000000000000, NATIVE_DENOM))];
    let money_bags = "owner_id".to_string();

    // instantiate with owner, then add treasury
    let msg = InstantiateMsg {
        denom: NATIVE_DENOM.to_string(),
        owner_id: None,
        gas_action_fee: None,
        gas_fraction: None,
        agent_nomination_duration: Some(360),
        cw_rules_addr: "todo".to_string(),
        gas_base_fee: None,
    };
    let res_init = store
        .instantiate(deps.as_mut(), mock_env(), info.clone(), msg)
        .unwrap();
    assert!(res_init.messages.is_empty());

    let payload = ExecuteMsg::UpdateSettings {
        paused: None,
        owner_id: None,
        // treasury_id: Some(money_bags.clone()),
        agent_fee: None,
        min_tasks_per_agent: None,
        agents_eject_threshold: None,
        gas_fraction: None,
        proxy_callback_gas: None,
        slot_granularity_time: None,
        gas_base_fee: None,
        gas_action_fee: None,
    };
    let info_settings = mock_info("owner_id", &coins(0, "meow"));
    let res_exec = store
        .execute(deps.as_mut(), mock_env(), info_settings, payload)
        .unwrap();
    assert!(res_exec.messages.is_empty());

    // try to move funds with greater amount than native available
    let msg_move_fail = ExecuteMsg::MoveBalances {
        balances: spensive_bal,
        account_id: money_bags.clone(),
    };
    let res_fail = store.execute(deps.as_mut(), mock_env(), info.clone(), msg_move_fail);
    match res_fail {
        Err(ContractError::CustomError { .. }) => {}
        _ => panic!("Must return custom not enough funds error"),
    }

    // try to move native available funds
    let msg_move = ExecuteMsg::MoveBalances {
        balances: exist_bal,
        account_id: money_bags,
    };
    let res_exec = store
        .execute(deps.as_mut(), mock_env(), info.clone(), msg_move)
        .unwrap();
    assert!(!res_exec.messages.is_empty());

    // it worked, let's query the state
    let res_bal = store
        .query(deps.as_ref(), mock_env(), QueryMsg::GetBalances {})
        .unwrap();
    let balances: GetBalancesResponse = from_binary(&res_bal).unwrap();
    assert_eq!(
        vec![coin(199999998, NATIVE_DENOM), coin(1000, "meow")],
        balances.available_balance.native
    );
}

// // TODO: Setup CW20 logic / balances!
// #[test]
// fn move_balances_cw() {
//     let mut deps = mock_dependencies_with_balance(&coins(200000000, NATIVE_DENOM));
//     let info = mock_info("owner_id", &vec![Balance::Cw20(1000, "meow")]);
//     let money_bags = Addr::unchecked("money_bags");
//     let exist_bal = vec![Balance::from(coins(2, NATIVE_DENOM))];
//     let spensive_bal = vec![Balance::from(coins(2000000000000, NATIVE_DENOM))];
//     let non_exist_bal = vec![Balance::from(coins(2, "shrute_bucks"))];

//     // instantiate with owner, then add treasury
//     let msg = InstantiateMsg { owner_id: None };
//     let res_init = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
//     assert!(res_init.messages.is_empty());

//     let payload = ExecuteMsg::UpdateSettings {
//         paused: None,
//         owner_id: None,
//         treasury_id: Some(money_bags.clone()),
//         agent_fee: None,
//         agent_task_ratio: None,
//         agents_eject_threshold: None,
//         gas_price: None,
//         proxy_callback_gas: None,
//         slot_granularity: None,
//     };
//     let res_exec = execute(deps.as_mut(), mock_env(), info.clone(), payload).unwrap();
//     assert!(res_exec.messages.is_empty());

//     // try to move funds with greater amount than cw available
//     let msg_move_fail = ExecuteMsg::MoveBalances { balances: spensive_bal, account_id: money_bags.clone() };
//     let res_fail = execute(deps.as_mut(), mock_env(), info.clone(), msg_move_fail);
//     match res_fail {
//         Err(ContractError::CustomError { .. }) => {}
//         _ => panic!("Must return custom not enough funds error"),
//     }

//     // try to move cw available funds
//     // // do the right thing
//     // let res_exec = execute(deps.as_mut(), mock_env(), info.clone(), payload).unwrap();
//     // assert!(!res_exec.messages.is_empty());

//     // // it worked, let's query the state
//     // let res = query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap();
//     // let value: ConfigResponse = from_binary(&res).unwrap();
//     // println!("CONFIG {:?}", value);
//     // assert_eq!(true, value.paused);
//     // assert_eq!(info.sender, value.owner_id);
// }

#[test]
fn test_get_state() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let state: StdResult<CwCroncatResponse> = app.wrap().query_wasm_smart(
        &contract_addr.clone(),
        &QueryMsg::GetState {
            from_index: None,
            limit: None,
        },
    );
    assert!(state.is_ok());
    let state = state.unwrap();

    assert_eq!(state.config.paused, false);
    assert_eq!(state.config.owner_id.as_str(), ADMIN);
    assert_eq!(state.config.agent_fee, 5);
    assert_eq!(state.config.gas_base_fee, 300_000);
    assert_eq!(state.config.gas_action_fee, 130_000);
    assert_eq!(state.config.proxy_callback_gas, 3);
    assert!(state.agent_active_queue.is_empty());
    assert!(state.agent_pending_queue.is_empty());
    assert!(state.agents.is_empty());
    assert!(state.tasks.is_empty());
    assert_eq!(state.task_total, Uint64::zero());
    assert!(state.time_slots.is_empty());
    assert!(state.block_slots.is_empty());
    assert!(state.tasks_with_rules.is_empty());
    assert_eq!(state.tasks_with_rules_total, Uint64::zero());
    assert!(state.time_slots_rules.is_empty());
    assert!(state.block_slots_rules.is_empty());
    assert_eq!(state.reply_index, Uint64::zero());
    assert_eq!(state.agent_nomination_begin_time, None);
    assert_eq!(
        state.balancer_mode,
        RoundRobinBalancerModeResponse::ActivationOrder
    );
    assert!(state.balances.is_empty());

    // Create a task
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&ExecuteMsg::WithdrawReward {}).unwrap(),
        funds: coins(1, NATIVE_DENOM),
    });

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: Some(Boundary::Height {
                start: None,
                end: None,
            }),
            stop_on_fail: false,
            actions: vec![Action {
                msg,
                gas_limit: Some(250_000),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
        },
    };

    // create a task
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &create_task_msg,
        &coins(525000, NATIVE_DENOM),
    )
    .unwrap();

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();
    // in pending queue
    app.execute_contract(Addr::unchecked(ANYONE), contract_addr.clone(), &msg, &[])
        .unwrap();

    // might need block advancement
    app.update_block(add_little_time);

    let state: StdResult<CwCroncatResponse> = app.wrap().query_wasm_smart(
        &contract_addr.clone(),
        &QueryMsg::GetState {
            from_index: None,
            limit: None,
        },
    );
    assert!(state.is_ok());
    let state = state.unwrap();

    let task_id_str =
        "1032a37c92801f73c75816bddb4f0db8516baeeeacd6a2c225f0a6a54c96732e".to_string();

    assert_eq!(state.agent_active_queue.len(), 1);
    assert_eq!(state.agent_active_queue[0].as_str(), AGENT0);
    assert_eq!(state.agent_pending_queue.len(), 1);
    assert_eq!(state.agent_pending_queue[0].as_str(), ANYONE);
    assert_eq!(state.agents.len(), 1);
    assert_eq!(state.tasks.len(), 1);
    assert_eq!(state.tasks[0].task_hash, task_id_str);
    assert_eq!(state.task_total, Uint64::from(1u64));
    assert!(state.time_slots.is_empty());
    assert_eq!(state.block_slots.len(), 1);
    assert!(state.tasks_with_rules.is_empty());
    assert_eq!(state.tasks_with_rules_total, Uint64::zero());
    assert!(state.time_slots_rules.is_empty());
    assert!(state.block_slots_rules.is_empty());
    assert_eq!(state.reply_index, Uint64::zero());
    assert!(state.agent_nomination_begin_time.is_some());
    assert!(state.balances.is_empty());
}
