use crate::contract::{
    GAS_ACTION_FEE, GAS_ADJUSTMENT_NUMERATOR_DEFAULT, GAS_BASE_FEE, GAS_DENOMINATOR,
    GAS_NUMERATOR_DEFAULT,
};

use crate::tests::helpers::{
    add_1000_blocks, add_little_time, add_one_duration_of_time, cw4_template, proper_instantiate,
    AGENT1, AGENT2, AGENT3,
};
use crate::ContractError;
use cosmwasm_std::{
    coin, coins, to_binary, Addr, BankMsg, Coin, CosmosMsg, StakingMsg, StdResult, Uint128, WasmMsg,
};
use cw20::Cw20Coin;
use cw_croncat_core::error::CoreError;
use cw_croncat_core::msg::{
    AgentResponse, AgentTaskResponse, ExecuteMsg, GetAgentIdsResponse, QueryMsg, TaskRequest,
    TaskResponse, TaskWithQueriesResponse,
};
use cw_croncat_core::types::{Action, Boundary, Interval, Transform};
use cw_multi_test::Executor;
use cw_rules_core::types::{CroncatQuery, HasBalanceGte};
use dao_core::state::ProposalModule;
use dao_voting::proposal::SingleChoiceProposeMsg;
use generic_query::{GenericQuery, PathToValue, ValueIndex, ValueOrdering};
use smart_query::{SmartQueries, SmartQuery, SmartQueryHead};

use super::helpers::{
    proper_instantiate_with_dao, ADMIN, AGENT0, AGENT_BENEFICIARY, ANYONE, NATIVE_DENOM,
};

#[test]
fn proxy_call_fail_cases() -> StdResult<()> {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();
    let proxy_call_msg = ExecuteMsg::ProxyCall { task_hash: None };
    let validator = String::from("you");
    let amount = coin(3, NATIVE_DENOM);
    let stake = StakingMsg::Delegate { validator, amount };
    let msg: CosmosMsg = stake.clone().into();

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
                gas_limit: Some(150_000),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
        },
    };

    // Must attach funds
    let res_err = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &proxy_call_msg,
            &coins(300010, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::CustomError {
            val: "Must not attach funds".to_string()
        },
        res_err.downcast().unwrap()
    );

    // AgentNotRegistered
    let res_err = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &proxy_call_msg,
            &vec![],
        )
        .unwrap_err();
    assert_eq!(
        ContractError::AgentNotRegistered {},
        res_err.downcast().unwrap()
    );

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();

    // Create task paused
    let change_settings_msg = ExecuteMsg::UpdateSettings {
        paused: Some(true),
        owner_id: None,
        // treasury_id: None,
        agent_fee: None,
        min_tasks_per_agent: None,
        agents_eject_threshold: None,
        gas_price: None,
        proxy_callback_gas: None,
        slot_granularity_time: None,
        gas_base_fee: None,
        gas_action_fee: None,
        gas_query_fee: None,
        gas_wasm_query_fee: None,
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &change_settings_msg,
        &vec![],
    )
    .unwrap();

    let agent_before_proxy_call: Option<AgentResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetAgent {
                account_id: String::from(AGENT0),
            },
        )
        .unwrap();

    // proxy_call in the next block
    app.update_block(add_little_time);
    let res_err = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &proxy_call_msg,
            &vec![],
        )
        .unwrap_err();
    assert_eq!(
        ContractError::CustomError {
            val: "Contract paused".to_string()
        },
        res_err.downcast().unwrap()
    );
    let agent_after_proxy_call: Option<AgentResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetAgent {
                account_id: String::from(AGENT0),
            },
        )
        .unwrap();
    // last_executed_slot for this agent didn't change since proxy_call failed
    assert!(
        agent_after_proxy_call.unwrap().last_executed_slot
            == agent_before_proxy_call.unwrap().last_executed_slot
    );

    // Set it back
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::UpdateSettings {
            paused: Some(false),
            owner_id: None,
            // treasury_id: None,
            agent_fee: None,
            min_tasks_per_agent: None,
            agents_eject_threshold: None,
            gas_price: None,
            proxy_callback_gas: None,
            slot_granularity_time: None,
            gas_base_fee: None,
            gas_action_fee: None,
            gas_query_fee: None,
            gas_wasm_query_fee: None,
        },
        &vec![],
    )
    .unwrap();

    // create task, so any slot actually exists
    let res = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &create_task_msg,
            &coins(315006, NATIVE_DENOM),
        )
        .unwrap();
    // Assert task hash is returned as part of event attributes
    let mut has_created_hash: bool = false;
    for e in res.events {
        for a in e.attributes {
            if a.key == "task_hash" && a.value.len() > 0 {
                has_created_hash = true;
            }
        }
    }
    assert!(has_created_hash);

    // The slot doesn't have tasks
    let res_no_tasks = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &proxy_call_msg,
            &vec![],
        )
        .unwrap();
    assert!(res_no_tasks.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "has_task" && attr.value == "false")));

    // NOTE: Unless there's a way to fake a task getting removed but hash remains in slot,
    // this coverage is not mockable. There literally shouldn't be any code that allows
    // this scenario to happen since all slot/task removal cases are covered
    // // delete the task so we test leaving an empty slot
    // app.execute_contract(
    //     Addr::unchecked(ANYONE),
    //     contract_addr.clone(),
    //     &ExecuteMsg::RemoveTask {
    //         task_hash: task_id_str.clone(),
    //     },
    //     &vec![],
    // )
    // .unwrap();

    // // NoTaskFound
    // let res_err = app
    //     .execute_contract(
    //         Addr::unchecked(AGENT0),
    //         contract_addr.clone(),
    //         &proxy_call_msg,
    //         &vec![],
    //     )
    //     .unwrap_err();
    // assert_eq!(
    //     ContractError::NoTaskFound {},
    //     res_err.downcast().unwrap()
    // );

    // TODO: TestCov: Task balance too small

    Ok(())
}

// TODO: TestCov: Agent balance updated (send_base_agent_reward)
// TODO: TestCov: Total balance updated
#[test]
fn proxy_call_success() -> StdResult<()> {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();
    let proxy_call_msg = ExecuteMsg::ProxyCall { task_hash: None };
    let task_id_str =
        "c67daffe9c635cc899c44f4fa8a5c40ed38ad41c46c3e92cd395a6408162feb3".to_string();

    // Doing this msg since its the easiest to guarantee success in reply
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&ExecuteMsg::WithdrawReward {})?,
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
    let res = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &create_task_msg,
            &coins(525000, NATIVE_DENOM),
        )
        .unwrap();
    // Assert task hash is returned as part of event attributes
    let mut has_created_hash: bool = false;
    for e in res.events {
        for a in e.attributes {
            if a.key == "task_hash" && a.value.len() > 0 {
                has_created_hash = true;
            }
        }
    }
    assert!(has_created_hash);

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();
    app.execute_contract(
        Addr::unchecked(contract_addr.clone()),
        contract_addr.clone(),
        &msg,
        &[],
    )
    .unwrap();

    // might need block advancement?!
    app.update_block(add_little_time);

    let agent_before_proxy_call: Option<AgentResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetAgent {
                account_id: String::from(AGENT0),
            },
        )
        .unwrap();

    // execute proxy_call
    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &proxy_call_msg,
            &vec![],
        )
        .unwrap();

    let agent_after_proxy_call: Option<AgentResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetAgent {
                account_id: String::from(AGENT0),
            },
        )
        .unwrap();
    // Check that last_executed_slot for this agent increased after proxy_call
    assert!(
        agent_after_proxy_call.unwrap().last_executed_slot
            == agent_before_proxy_call.unwrap().last_executed_slot + 1
    );

    let mut has_required_attributes: bool = true;
    let mut has_submsg_method: bool = false;
    let mut has_reply_success: bool = false;
    let attributes = vec![
        ("method", "proxy_call"),
        ("agent", AGENT0),
        ("slot_id", "12346"),
        ("slot_kind", "Block"),
        ("task_hash", task_id_str.as_str().clone()),
    ];

    // check all attributes are covered in response, and match the expected values
    for (k, v) in attributes.iter() {
        let mut attr_key: Option<String> = None;
        let mut attr_value: Option<String> = None;
        for e in res.clone().events {
            for a in e.attributes {
                if e.ty == "wasm" && a.clone().key == k.to_string() && attr_key.is_none() {
                    attr_key = Some(a.clone().key);
                    attr_value = Some(a.clone().value);
                }
                if e.ty == "wasm"
                    && a.clone().key == "method"
                    && a.clone().value == "withdraw_agent_balance"
                {
                    has_submsg_method = true;
                }
                if e.ty == "reply" && a.clone().key == "mode" && a.clone().value == "handle_success"
                {
                    has_reply_success = true;
                }
            }
        }

        // flip bool if none found, or value doesnt match
        if let Some(_key) = attr_key {
            if let Some(value) = attr_value {
                if v.to_string() != value {
                    println!("{} - {}", v.to_string(), value);
                    has_required_attributes = false;
                }
            } else {
                has_required_attributes = false;
            }
        } else {
            has_required_attributes = false;
        }
    }
    assert!(has_required_attributes);
    assert!(has_submsg_method);
    assert!(has_reply_success);

    Ok(())
}

#[test]
fn proxy_call_no_task_and_withdraw() -> StdResult<()> {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let to_address = String::from("not_you");
    let amount = coin(1000, "atom");
    let send = BankMsg::Send {
        to_address,
        amount: vec![amount],
    };
    let msg: CosmosMsg = send.clone().into();
    let gas_limit = 150_000;

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg,
                gas_limit: Some(gas_limit),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
        },
    };
    let gas_for_one = GAS_BASE_FEE + gas_limit;
    let amount_for_one_task = gas_for_one * GAS_ADJUSTMENT_NUMERATOR_DEFAULT / GAS_DENOMINATOR
        * GAS_NUMERATOR_DEFAULT
        / GAS_DENOMINATOR;
    let agent_fee = amount_for_one_task * 5 / 100;
    let amount_with_fee = gas_limit + agent_fee + 1000;
    // create a task
    let res = app.execute_contract(
        Addr::unchecked(ANYONE),
        contract_addr.clone(),
        &create_task_msg,
        &coins(u128::from(amount_with_fee * 2), "atom"),
    );
    assert!(res.is_ok());

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();

    app.update_block(add_little_time);

    let res = app.execute_contract(
        Addr::unchecked(AGENT0),
        contract_addr.clone(),
        &ExecuteMsg::ProxyCall { task_hash: None },
        &[],
    );
    assert!(res.is_ok());

    // Call proxy_call when there is no task, should fail
    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &ExecuteMsg::ProxyCall { task_hash: None },
            &[],
        )
        .unwrap();
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "has_task" && attr.value == "false")));

    let beneficiary_balance_before_proxy_call = app
        .wrap()
        .query_balance(AGENT_BENEFICIARY, NATIVE_DENOM)
        .unwrap();
    // Agent withdraws the reward
    let res = app.execute_contract(
        Addr::unchecked(AGENT0),
        contract_addr.clone(),
        &ExecuteMsg::WithdrawReward {},
        &[],
    );
    assert!(res.is_ok());
    let beneficiary_balance_after_proxy_call = app
        .wrap()
        .query_balance(AGENT_BENEFICIARY, NATIVE_DENOM)
        .unwrap();
    assert_eq!(
        (beneficiary_balance_after_proxy_call.amount
            - beneficiary_balance_before_proxy_call.amount)
            .u128(),
        ((amount_for_one_task + agent_fee) as u128)
    );

    Ok(())
}

#[test]
fn proxy_callback_fail_cases() -> StdResult<()> {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();
    let proxy_call_msg = ExecuteMsg::ProxyCall { task_hash: None };
    let task_id_str =
        "0efd342012e0e2791902f167d9b87547fd73064690a8dc81d612704c89d6c77c".to_string();

    // Doing this msg since its the easiest to guarantee success in reply
    let validator = String::from("you");
    let amount = coin(3, NATIVE_DENOM);
    let stake = StakingMsg::Delegate { validator, amount };
    let msg: CosmosMsg = stake.clone().into();

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: Some(Boundary::Height {
                start: None,
                end: Some(12347_u64.into()),
            }),
            stop_on_fail: true,
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
    let res = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &create_task_msg,
            &coins(128338, NATIVE_DENOM),
        )
        .unwrap();
    // Assert task hash is returned as part of event attributes
    let mut has_created_hash: bool = false;
    for e in res.events {
        for a in e.attributes {
            if a.key == "task_hash" && a.value.len() > 0 {
                has_created_hash = true;
            }
        }
    }
    assert!(has_created_hash);

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();
    app.execute_contract(
        Addr::unchecked(contract_addr.clone()),
        contract_addr.clone(),
        &msg,
        &[],
    )
    .unwrap();

    // might need block advancement?!
    app.update_block(add_little_time);

    // execute proxy_call - STOP ON FAIL
    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &proxy_call_msg,
            &vec![],
        )
        .unwrap();
    let mut has_required_attributes: bool = true;
    let mut has_submsg_method: bool = false;
    let mut has_reply_success: bool = false;
    let attributes = vec![
        ("method", "remove_task"), // the last method
        ("slot_id", "12346"),
        ("slot_kind", "Block"),
        ("task_hash", task_id_str.as_str().clone()),
    ];

    // check all attributes are covered in response, and match the expected values
    for (k, v) in attributes.iter() {
        let mut attr_key: Option<String> = None;
        let mut attr_value: Option<String> = None;
        for e in res.clone().events {
            for a in e.attributes.clone() {
                if e.ty == "wasm" && a.clone().key == k.to_string() {
                    attr_key = Some(a.clone().key);
                    attr_value = Some(a.clone().value);
                }
                if e.ty == "transfer" && a.clone().key == "amount" && a.clone().value == "93688atom"
                // task didn't pay for the failed execution
                {
                    has_submsg_method = true;
                }
                if e.ty == "reply" && a.clone().key == "mode" && a.clone().value == "handle_failure"
                {
                    has_reply_success = true;
                }
            }
        }

        // flip bool if none found, or value doesnt match
        if let Some(_key) = attr_key {
            if let Some(value) = attr_value {
                if v.to_string() != value {
                    println!("{} - {}", v.to_string(), value);
                    has_required_attributes = false;
                }
            } else {
                has_required_attributes = false;
            }
        } else {
            has_required_attributes = false;
        }
    }
    assert!(has_required_attributes);
    assert!(has_submsg_method);
    assert!(has_reply_success);

    // let task_id_str =
    //     "ce7f88df7816b4cf2d0cd882f189eb81ad66e4a9aabfc1eb5ba2189d73f9929b".to_string();

    // Doing this msg since its the easiest to guarantee success in reply
    let validator = String::from("you");
    let amount = coin(3, NATIVE_DENOM);
    let stake = StakingMsg::Delegate { validator, amount };
    let msg: CosmosMsg = stake.clone().into();

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: Some(Boundary::Height {
                start: None,
                end: Some(12347_u64.into()),
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

    // create the task again
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &create_task_msg,
        &coins(525006, NATIVE_DENOM),
    )
    .unwrap();

    // might need block advancement?!
    app.update_block(add_little_time);
    app.update_block(add_little_time);

    // execute proxy_call - TASK ENDED
    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &proxy_call_msg,
            &vec![],
        )
        .unwrap();
    let mut has_required_attributes: bool = true;
    let mut has_submsg_method: bool = false;
    let mut has_reply_success: bool = false;
    let attributes = vec![
        ("method", "remove_task"), // the last method
        ("ended_task", task_id_str.as_str().clone()),
    ];

    // check all attributes are covered in response, and match the expected values
    for (k, v) in attributes.iter() {
        let mut attr_key: Option<String> = None;
        let mut attr_value: Option<String> = None;
        for e in res.clone().events {
            for a in e.attributes {
                if e.ty == "wasm" && a.clone().key == k.to_string() {
                    attr_key = Some(a.clone().key);
                    attr_value = Some(a.clone().value);
                }
                if e.ty == "transfer"
                    && a.clone().key == "amount"
                    && a.clone().value == "490356atom"
                // task didn't pay for the failed execution
                {
                    has_submsg_method = true;
                }
                if e.ty == "reply" && a.clone().key == "mode" && a.clone().value == "handle_failure"
                {
                    has_reply_success = true;
                }
            }
        }

        // flip bool if none found, or value doesnt match
        if let Some(_key) = attr_key {
            if let Some(value) = attr_value {
                if v.to_string() != value {
                    has_required_attributes = false;
                }
            } else {
                has_required_attributes = false;
            }
        } else {
            has_required_attributes = false;
        }
    }
    assert!(has_required_attributes);
    assert!(has_submsg_method);
    assert!(has_reply_success);

    Ok(())
}

#[test]
fn proxy_callback_block_slots() -> StdResult<()> {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();
    let proxy_call_msg = ExecuteMsg::ProxyCall { task_hash: None };
    let task_id_str =
        "c67daffe9c635cc899c44f4fa8a5c40ed38ad41c46c3e92cd395a6408162feb3".to_string();

    // Doing this msg since its the easiest to guarantee success in reply
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&ExecuteMsg::WithdrawReward {})?,
        funds: coins(1, NATIVE_DENOM),
    });

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
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
    let res = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &create_task_msg,
            &coins(525000, NATIVE_DENOM),
        )
        .unwrap();
    // Assert task hash is returned as part of event attributes
    let mut has_created_hash: bool = false;
    for e in res.events {
        for a in e.attributes {
            if a.key == "task_hash" && a.value.len() > 0 {
                has_created_hash = true;
            }
        }
    }
    assert!(has_created_hash);

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();
    app.execute_contract(
        Addr::unchecked(contract_addr.clone()),
        contract_addr.clone(),
        &msg,
        &[],
    )
    .unwrap();

    // might need block advancement?!
    app.update_block(add_little_time);

    // execute proxy_call
    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &proxy_call_msg,
            &vec![],
        )
        .unwrap();
    let mut has_required_attributes: bool = true;
    let mut has_submsg_method: bool = false;
    let mut has_reply_success: bool = false;
    let attributes = vec![
        ("method", "proxy_callback"),
        ("slot_id", "12347"),
        ("slot_kind", "Block"),
        ("task_hash", task_id_str.as_str().clone()),
    ];

    // check all attributes are covered in response, and match the expected values
    for (k, v) in attributes.iter() {
        let mut attr_key: Option<String> = None;
        let mut attr_value: Option<String> = None;
        for e in res.clone().events {
            for a in e.attributes {
                if e.ty == "wasm" && a.clone().key == k.to_string() {
                    attr_key = Some(a.clone().key);
                    attr_value = Some(a.clone().value);
                }
                if e.ty == "wasm"
                    && a.clone().key == "method"
                    && a.clone().value == "withdraw_agent_balance"
                {
                    has_submsg_method = true;
                }
                if e.ty == "reply" && a.clone().key == "mode" && a.clone().value == "handle_success"
                {
                    has_reply_success = true;
                }
            }
        }

        // flip bool if none found, or value doesnt match
        if let Some(_key) = attr_key {
            if let Some(value) = attr_value {
                if v.to_string() != value {
                    println!("{} - {}", v.to_string(), value);
                    has_required_attributes = false;
                }
            } else {
                has_required_attributes = false;
            }
        } else {
            has_required_attributes = false;
        }
    }
    assert!(has_required_attributes);
    assert!(has_submsg_method);
    assert!(has_reply_success);

    Ok(())
}

#[test]
fn proxy_callback_time_slots() -> StdResult<()> {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();
    let proxy_call_msg = ExecuteMsg::ProxyCall { task_hash: None };
    let task_id_str =
        "48a8dec99cd112818db4a74ef3e84fb10d524203aaecbaccd28565627062f65f".to_string();

    // Doing this msg since its the easiest to guarantee success in reply
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&ExecuteMsg::WithdrawReward {})?,
        funds: coins(1, NATIVE_DENOM),
    });

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Cron("0 * * * * *".to_string()),
            boundary: None,
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
    let res = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &create_task_msg,
            &coins(525000, NATIVE_DENOM),
        )
        .unwrap();

    // Assert task hash is returned as part of event attributes
    let mut has_created_hash: bool = false;
    for e in res.events {
        for a in e.attributes {
            if a.key == "task_hash" && a.value.len() > 0 {
                has_created_hash = true;
            }
        }
    }
    assert!(has_created_hash);

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();
    app.execute_contract(
        Addr::unchecked(contract_addr.clone()),
        contract_addr.clone(),
        &msg,
        &[],
    )
    .unwrap();

    // might need block advancement?!
    app.update_block(add_one_duration_of_time);

    // execute proxy_call
    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &proxy_call_msg,
            &vec![],
        )
        .unwrap();
    let mut has_required_attributes: bool = true;
    let mut has_submsg_method: bool = false;
    let mut has_reply_success: bool = false;
    let attributes = vec![
        ("method", "proxy_callback"),
        ("slot_id", "1571797860000000000"),
        ("slot_kind", "Cron"),
        ("task_hash", task_id_str.as_str().clone()),
    ];

    // check all attributes are covered in response, and match the expected values
    for (k, v) in attributes.iter() {
        let mut attr_key: Option<String> = None;
        let mut attr_value: Option<String> = None;
        for e in res.clone().events {
            for a in e.attributes {
                if e.ty == "wasm" && a.clone().key == k.to_string() {
                    attr_key = Some(a.clone().key);
                    attr_value = Some(a.clone().value);
                }
                if e.ty == "wasm"
                    && a.clone().key == "method"
                    && a.clone().value == "withdraw_agent_balance"
                {
                    has_submsg_method = true;
                }
                if e.ty == "reply" && a.clone().key == "mode" && a.clone().value == "handle_success"
                {
                    has_reply_success = true;
                }
            }
        }

        // flip bool if none found, or value doesnt match
        if let Some(_key) = attr_key {
            if let Some(value) = attr_value {
                if v.to_string() != value {
                    println!("{} - {}", v.to_string(), value);

                    has_required_attributes = false;
                }
            } else {
                has_required_attributes = false;
            }
        } else {
            has_required_attributes = false;
        }
    }
    assert!(has_required_attributes);
    assert!(has_submsg_method);
    assert!(has_reply_success);

    Ok(())
}

#[test]
fn proxy_call_several_tasks() -> StdResult<()> {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();
    let proxy_call_msg = ExecuteMsg::ProxyCall { task_hash: None };

    // Doing this msg since its the easiest to guarantee success in reply
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&ExecuteMsg::WithdrawReward {})?,
        funds: coins(1, NATIVE_DENOM),
    });

    let msg2 = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&ExecuteMsg::WithdrawReward {})?,
        funds: coins(2, NATIVE_DENOM),
    });

    let msg3 = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&ExecuteMsg::WithdrawReward {})?,
        funds: coins(3, NATIVE_DENOM),
    });

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
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

    let create_task_msg2 = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: msg2,
                gas_limit: Some(250_000),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
        },
    };

    let create_task_msg3 = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: msg3,
                gas_limit: Some(250_000),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
        },
    };

    // create two tasks in the same block
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &create_task_msg,
        &coins(525000, NATIVE_DENOM),
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &create_task_msg2,
        &coins(525000, NATIVE_DENOM),
    )
    .unwrap();

    // the third task is created in another block
    app.update_block(add_little_time);

    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &create_task_msg3,
        &coins(525000, NATIVE_DENOM),
    )
    .unwrap();

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();
    app.execute_contract(
        Addr::unchecked(contract_addr.clone()),
        contract_addr.clone(),
        &msg,
        &[],
    )
    .unwrap();

    // need block advancement
    app.update_block(add_little_time);

    // execute proxy_call's
    let res = app.execute_contract(
        Addr::unchecked(AGENT0),
        contract_addr.clone(),
        &proxy_call_msg,
        &vec![],
    );
    assert!(res.is_ok());

    let res = app.execute_contract(
        Addr::unchecked(AGENT0),
        contract_addr.clone(),
        &proxy_call_msg,
        &vec![],
    );
    assert!(res.is_ok());

    let res = app.execute_contract(
        Addr::unchecked(AGENT0),
        contract_addr.clone(),
        &proxy_call_msg,
        &vec![],
    );
    assert!(res.is_ok());
    Ok(())
}

#[test]
fn test_proxy_call_with_bank_message() -> StdResult<()> {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let to_address = String::from("not_you");
    let amount = coin(1000, NATIVE_DENOM);
    let send = BankMsg::Send {
        to_address,
        amount: vec![amount],
    };
    let msg: CosmosMsg = send.clone().into();
    let gas_limit = 150_000;

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg,
                gas_limit: Some(gas_limit),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
        },
    };
    let amount_for_one_task =
        gas_limit + gas_limit.checked_mul(5).unwrap().checked_div(100).unwrap() + 1000;
    // create a task
    let res = app.execute_contract(
        Addr::unchecked(ANYONE),
        contract_addr.clone(),
        &create_task_msg,
        &coins(u128::from(amount_for_one_task * 2), NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();

    app.update_block(add_little_time);

    let res = app.execute_contract(
        Addr::unchecked(AGENT0),
        contract_addr.clone(),
        &ExecuteMsg::ProxyCall { task_hash: None },
        &[],
    );

    assert!(res.is_ok());
    Ok(())
}
#[test]
fn test_proxy_call_with_bank_message_should_fail() -> StdResult<()> {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let to_address = String::from("not_you");
    let amount = coin(600_000, NATIVE_DENOM);
    let send = BankMsg::Send {
        to_address,
        amount: vec![amount],
    };
    let msg: CosmosMsg = send.clone().into();
    let gas_limit: u64 = 150_000;
    let agent_fee = gas_limit.checked_mul(5).unwrap().checked_div(100).unwrap();

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg,
                gas_limit: Some(gas_limit),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
        },
    };
    // create 1 token off task
    let amount_for_one_task = gas_limit + agent_fee;
    // create a task
    let res = app.execute_contract(
        Addr::unchecked(ANYONE),
        contract_addr.clone(),
        &create_task_msg,
        &coins(u128::from(amount_for_one_task * 2), NATIVE_DENOM),
    );
    assert!(res.is_err()); //Will fail, abount of send > then task.total_deposit

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();

    app.update_block(add_little_time);

    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &ExecuteMsg::ProxyCall { task_hash: None },
            &[],
        )
        .unwrap();
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "has_task" && attr.value == "false")));

    Ok(())
}

#[test]
fn test_multi_action() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let addr1 = String::from("addr1");
    let addr2 = String::from("addr2");
    let amount = coins(3, NATIVE_DENOM);
    let send = BankMsg::Send {
        to_address: addr1,
        amount,
    };
    let msg1: CosmosMsg = send.into();
    let amount = coins(4, NATIVE_DENOM);
    let send = BankMsg::Send {
        to_address: addr2,
        amount,
    };
    let msg2: CosmosMsg = send.into();

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Once,
            boundary: None,
            stop_on_fail: false,
            actions: vec![
                Action {
                    msg: msg1,
                    gas_limit: None,
                },
                Action {
                    msg: msg2,
                    gas_limit: None,
                },
            ],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
        },
    };
    let gas_limit = GAS_ACTION_FEE;
    let agent_fee = gas_limit.checked_mul(5).unwrap().checked_div(100).unwrap();
    let amount_for_one_task = (gas_limit * 2) + agent_fee * 2 + 3 + 4; // + 3 + 4 atoms sent

    // create a task
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &create_task_msg,
        &coins(u128::from(amount_for_one_task), NATIVE_DENOM),
    )
    .unwrap();

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();

    app.update_block(add_little_time);

    let proxy_call_msg = ExecuteMsg::ProxyCall { task_hash: None };
    let res = app.execute_contract(
        Addr::unchecked(AGENT0),
        contract_addr.clone(),
        &proxy_call_msg,
        &[],
    );
    assert!(res.is_ok());
}

#[test]
fn test_balance_changes() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let addr1 = String::from("addr1");
    let addr2 = String::from("addr2");
    let amount = coins(3, NATIVE_DENOM);
    let send = BankMsg::Send {
        to_address: addr1,
        amount,
    };
    let msg1: CosmosMsg = send.into();
    let amount = coins(4, NATIVE_DENOM);
    let send = BankMsg::Send {
        to_address: addr2,
        amount,
    };
    let msg2: CosmosMsg = send.into();

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Once,
            boundary: None,
            stop_on_fail: false,
            actions: vec![
                Action {
                    msg: msg1,
                    gas_limit: None,
                },
                Action {
                    msg: msg2,
                    gas_limit: None,
                },
            ],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
        },
    };
    let gas_for_one = GAS_BASE_FEE + (GAS_ACTION_FEE * 2);
    let agent_fee = gas_for_one * 5 / 100;
    let extra = 50; // extra for checking refunds at task removal
    let amount_for_one_task = (gas_for_one + agent_fee) * GAS_ADJUSTMENT_NUMERATOR_DEFAULT
        / GAS_DENOMINATOR
        * GAS_NUMERATOR_DEFAULT
        / GAS_DENOMINATOR
        + 3
        + 4
        + extra; // + 3 + 4 atoms sent

    // create a task
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &create_task_msg,
        &coins(u128::from(amount_for_one_task), NATIVE_DENOM),
    )
    .unwrap();

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();

    app.update_block(add_little_time);

    // checking changes to contract balances and to the task creator
    let contract_balance_before_proxy_call = app
        .wrap()
        .query_balance(&contract_addr, NATIVE_DENOM)
        .unwrap();
    let admin_balance_before_proxy_call = app.wrap().query_balance(ADMIN, NATIVE_DENOM).unwrap();
    let proxy_call_msg = ExecuteMsg::ProxyCall { task_hash: None };
    app.execute_contract(
        Addr::unchecked(AGENT0),
        contract_addr.clone(),
        &proxy_call_msg,
        &vec![],
    )
    .unwrap();
    let contract_balance_after_proxy_call = app
        .wrap()
        .query_balance(&contract_addr, NATIVE_DENOM)
        .unwrap();
    assert_eq!(
        contract_balance_after_proxy_call.amount,
        contract_balance_before_proxy_call.amount - Uint128::from(extra + 3 + 4)
    );
    let admin_balance_after_proxy_call = app.wrap().query_balance(ADMIN, NATIVE_DENOM).unwrap();
    assert_eq!(
        admin_balance_after_proxy_call.amount,
        admin_balance_before_proxy_call.amount + Uint128::from(extra)
    );

    // checking balances of recipients
    let balance_addr1 = app.wrap().query_balance("addr1", NATIVE_DENOM).unwrap();
    assert_eq!(
        balance_addr1,
        Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::from(3_u128),
        }
    );

    let balance_addr2 = app.wrap().query_balance("addr2", NATIVE_DENOM).unwrap();
    assert_eq!(
        balance_addr2,
        Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::from(4_u128),
        }
    );

    // checking balance of agent and contract after withdrawal
    let beneficary_balance_before_withdraw = app
        .wrap()
        .query_balance(AGENT_BENEFICIARY, NATIVE_DENOM)
        .unwrap();
    let contract_balance_before_withdraw = app
        .wrap()
        .query_balance(&contract_addr, NATIVE_DENOM)
        .unwrap();
    let withdraw_msg = ExecuteMsg::WithdrawReward {};
    app.execute_contract(
        Addr::unchecked(AGENT0),
        contract_addr.clone(),
        &withdraw_msg,
        &[],
    )
    .unwrap();
    let beneficary_balance_after_withdraw = app
        .wrap()
        .query_balance(AGENT_BENEFICIARY, NATIVE_DENOM)
        .unwrap();
    let contract_balance_after_withdraw = app
        .wrap()
        .query_balance(&contract_addr, NATIVE_DENOM)
        .unwrap();

    let expected_transfer_amount = Uint128::from(amount_for_one_task - extra - 3 - 4);
    assert_eq!(
        beneficary_balance_after_withdraw.amount,
        beneficary_balance_before_withdraw.amount + expected_transfer_amount
    );
    assert_eq!(
        contract_balance_after_withdraw.amount,
        contract_balance_before_withdraw.amount - expected_transfer_amount
    )
}

#[test]
fn test_no_reschedule_if_lack_balance() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let addr1 = String::from("addr1");
    let amount = coins(3, NATIVE_DENOM);
    let send = BankMsg::Send {
        to_address: addr1,
        amount,
    };
    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: send.into(),
                gas_limit: None,
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
        },
    };

    let gas_for_one = GAS_BASE_FEE + GAS_ACTION_FEE;
    let agent_fee = gas_for_one * 5 / 100;
    let extra = 50; // extra for checking nonzero task balance
    let amount_for_one_task = (gas_for_one + agent_fee) * GAS_ADJUSTMENT_NUMERATOR_DEFAULT
        / GAS_DENOMINATOR
        * GAS_NUMERATOR_DEFAULT
        / GAS_DENOMINATOR
        + 3; // + 3 atoms sent

    // create a task
    let resp = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &create_task_msg,
            &coins(u128::from(amount_for_one_task * 2 + extra - 3), "atom"),
        )
        .unwrap();

    let mut hash = String::new();
    for e in resp.events {
        for a in e.attributes {
            if a.key == "task_hash" && a.value.len() > 0 {
                hash = a.value;
            }
        }
    }

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();

    let proxy_call_msg = ExecuteMsg::ProxyCall { task_hash: None };
    // executing it two times
    app.update_block(add_little_time);
    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &proxy_call_msg,
            &vec![],
        )
        .unwrap();
    assert!(res.events.iter().any(|event| {
        event
            .attributes
            .iter()
            .any(|attr| attr.key == "method" && attr.value == "proxy_callback")
    }));

    let task: Option<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetTask { task_hash: hash },
        )
        .unwrap();
    assert_eq!(
        task.unwrap().total_deposit[0].amount,
        Uint128::from(
            (gas_for_one + agent_fee) * GAS_ADJUSTMENT_NUMERATOR_DEFAULT / GAS_DENOMINATOR
                * GAS_NUMERATOR_DEFAULT
                / GAS_DENOMINATOR
                + extra
        )
    );

    app.update_block(add_little_time);
    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &proxy_call_msg,
            &vec![],
        )
        .unwrap();
    assert!(res.events.iter().any(|event| {
        event
            .attributes
            .iter()
            .any(|attr| attr.key == "method" && attr.value == "proxy_callback")
    }));
    // third time it pays only base to agent
    // since "extra" is not enough to cover another task and it got removed
    let task: Option<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetTask {
                task_hash: "8fad55a869f129ba363786bd7f0ec698f1a59e2553ba7fdec408f1cd82326cd3"
                    .to_string(),
            },
        )
        .unwrap();
    assert!(task.is_none());
    app.update_block(add_little_time);
    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &proxy_call_msg,
            &vec![],
        )
        .unwrap();
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "has_task" && attr.value == "false")));
}

#[test]
fn test_complete_task_with_query() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let addr1 = String::from("addr1");
    let amount = coins(3, NATIVE_DENOM);
    let send = BankMsg::Send {
        to_address: addr1,
        amount,
    };
    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Once,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: send.clone().into(),
                gas_limit: None,
            }],
            queries: Some(vec![CroncatQuery::HasBalanceGte(HasBalanceGte {
                address: String::from("addr2"),
                required_balance: coins(1, NATIVE_DENOM).into(),
            })]),
            transforms: None,
            cw20_coins: vec![],
        },
    };

    let attached_balance = 900058;
    let resp = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &create_task_msg,
            &coins(attached_balance, NATIVE_DENOM),
        )
        .unwrap();

    let mut task_hash = String::new();
    for e in resp.events {
        for a in e.attributes {
            if a.key == "task_hash" && a.value.len() > 0 {
                task_hash = a.value;
            }
        }
    }
    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();

    app.update_block(add_little_time);

    let agent_tasks: Option<AgentTaskResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetAgentTasks {
                account_id: String::from(AGENT0),
            },
        )
        .unwrap();
    assert!(agent_tasks.is_none());

    let tasks_with_queries: Vec<TaskWithQueriesResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetTasksWithQueries {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(tasks_with_queries.len(), 1);
    app.send_tokens(
        Addr::unchecked(ADMIN),
        Addr::unchecked("addr2"),
        &coins(1, NATIVE_DENOM),
    )
    .unwrap();

    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &ExecuteMsg::ProxyCall {
                task_hash: Some(String::from(task_hash.clone())),
            },
            &[],
        )
        .unwrap();

    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "task_hash" && attr.value == task_hash)));
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "method" && attr.value == "proxy_callback")));

    let tasks_with_queries: Vec<TaskWithQueriesResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetTasksWithQueries {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert!(tasks_with_queries.is_empty());
}

#[test]
fn test_reschedule_task_with_queries() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let addr1 = String::from("addr1");
    let amount = coins(3, NATIVE_DENOM);
    let send = BankMsg::Send {
        to_address: addr1,
        amount,
    };
    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: send.clone().into(),
                gas_limit: None,
            }],
            queries: Some(vec![CroncatQuery::HasBalanceGte(HasBalanceGte {
                address: String::from("addr2"),
                required_balance: coins(1, NATIVE_DENOM).into(),
            })]),
            transforms: None,
            cw20_coins: vec![],
        },
    };

    let attached_balance = 100338 * 4;
    let create_task_resp = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &create_task_msg,
            &coins(attached_balance, NATIVE_DENOM),
        )
        .unwrap();
    let mut task_hash = String::new();
    for e in create_task_resp.events {
        for a in e.attributes {
            if a.key == "task_hash" && a.value.len() > 0 {
                task_hash = a.value;
            }
        }
    }

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();

    app.update_block(add_little_time);

    let agent_tasks: Option<AgentTaskResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetAgentTasks {
                account_id: String::from(AGENT0),
            },
        )
        .unwrap();
    assert!(agent_tasks.is_none());

    let tasks_with_queries: Vec<TaskWithQueriesResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetTasksWithQueries {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(tasks_with_queries.len(), 1);

    app.send_tokens(
        Addr::unchecked(ADMIN),
        Addr::unchecked("addr2"),
        &coins(1, NATIVE_DENOM),
    )
    .unwrap();

    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &ExecuteMsg::ProxyCall {
                task_hash: Some(String::from(task_hash.clone())),
            },
            &[],
        )
        .unwrap();
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "task_hash" && attr.value == task_hash.clone())));
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "method" && attr.value == "proxy_callback")));

    // Shouldn't affect tasks without queries
    let tasks_response: Vec<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetTasks {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert!(tasks_response.is_empty());

    // Run it a bunch of times successfully, until it's removed because the balance falls too low
    for _ in 1..12 {
        let _ = app
            .execute_contract(
                Addr::unchecked(AGENT0),
                contract_addr.clone(),
                &ExecuteMsg::ProxyCall {
                    task_hash: Some(task_hash.clone()),
                },
                &[],
            )
            .is_ok();
    }

    let tasks_with_queries: Vec<TaskWithQueriesResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetTasksWithQueries {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();

    assert!(tasks_with_queries.is_empty());
}

#[test]
fn tick() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    // Change settings, the agent can miss 1000 blocks
    let change_settings_msg = ExecuteMsg::UpdateSettings {
        paused: None,
        owner_id: None,
        agent_fee: None,
        min_tasks_per_agent: None,
        agents_eject_threshold: Some(1000), // allow to miss 1000 slots
        gas_action_fee: None,
        gas_query_fee: None,
        gas_wasm_query_fee: None,
        proxy_callback_gas: None,
        slot_granularity_time: None,
        gas_base_fee: None,
        gas_price: None,
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &change_settings_msg,
        &vec![],
    )
    .unwrap();

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();

    // Add 1001 blocks and call tick
    app.update_block(add_1000_blocks);
    app.update_block(add_little_time);
    let tick_msg = ExecuteMsg::Tick {};
    let res = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &tick_msg,
            &vec![],
        )
        .unwrap();

    // Check attributes
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "method" && attr.value == "tick")));
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "method" && attr.value == "unregister_agent")));
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "account_id" && attr.value == AGENT0)));
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "lifecycle" && attr.value == "tick_failure")));

    // The agent missed 1001 blocks and he was unregistered
    // Pending agents weren't deleted
    let agents: GetAgentIdsResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetAgentIds {})
        .unwrap();
    assert!(agents.active.is_empty());
    assert!(agents.pending.is_empty());

    // quick agent register
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();

    // Two agents added to the pending queue
    app.execute_contract(Addr::unchecked(AGENT1), contract_addr.clone(), &msg, &[])
        .unwrap();
    app.execute_contract(Addr::unchecked(AGENT2), contract_addr.clone(), &msg, &[])
        .unwrap();

    // need block advancement
    app.update_block(add_little_time);

    // Call tick
    // Not enough time passed to delete the agent
    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &tick_msg,
            &vec![],
        )
        .unwrap();
    // Check attributes
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "method" && attr.value == "tick")));
    assert!(!res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "method" && attr.value == "unregister_agent")));

    // The agent wasn't unregistered
    let agents: GetAgentIdsResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetAgentIds {})
        .unwrap();
    assert_eq!(agents.active.len(), 1);
    assert_eq!(agents.pending.len(), 2);

    // First pending agent wasn't nominated
    let err = app
        .execute_contract(
            Addr::unchecked(AGENT1),
            contract_addr.clone(),
            &ExecuteMsg::CheckInAgent {},
            &[],
        )
        .unwrap_err();
    assert_eq!(
        ContractError::CustomError {
            val: "Not accepting new agents".to_string()
        },
        err.downcast().unwrap()
    );

    // Add enough time and call tick
    app.update_block(add_1000_blocks);
    let res = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &tick_msg,
            &vec![],
        )
        .unwrap();

    // Check attributes
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "method" && attr.value == "tick")));
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "method" && attr.value == "unregister_agent")));
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "account_id" && attr.value == AGENT0)));
    assert!(!res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "lifecycle" && attr.value == "tick_failure")));

    // The agent missed 1001 blocks and he was unregistered
    // Pending agents weren't deleted
    let agents: GetAgentIdsResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetAgentIds {})
        .unwrap();
    assert!(agents.active.is_empty());
    assert_eq!(agents.pending.len(), 2);

    // First agent was nominated and can call CheckInAgent
    app.execute_contract(
        Addr::unchecked(AGENT1),
        contract_addr.clone(),
        &ExecuteMsg::CheckInAgent {},
        &[],
    )
    .unwrap();
    // Second agent wasn't nominated
    let err = app
        .execute_contract(
            Addr::unchecked(AGENT2),
            contract_addr.clone(),
            &ExecuteMsg::CheckInAgent {},
            &[],
        )
        .unwrap_err();
    assert_eq!(
        ContractError::CustomError {
            val: "Not accepting new agents".to_string()
        },
        err.downcast().unwrap()
    );
}

#[test]
fn tick_task() -> StdResult<()> {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let change_settings_msg = ExecuteMsg::UpdateSettings {
        paused: None,
        owner_id: None,
        agent_fee: None,
        min_tasks_per_agent: Some(1),
        agents_eject_threshold: Some(1000), // allow to miss 100 slots
        proxy_callback_gas: None,
        slot_granularity_time: None,
        gas_base_fee: None,
        gas_action_fee: None,
        gas_query_fee: None,
        gas_wasm_query_fee: None,
        gas_price: None,
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &change_settings_msg,
        &vec![],
    )
    .unwrap();

    // quick agent register
    let msg_register = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(
        Addr::unchecked(AGENT0),
        contract_addr.clone(),
        &msg_register,
        &[],
    )
    .unwrap();

    // Another agent
    app.execute_contract(
        Addr::unchecked(ANYONE),
        contract_addr.clone(),
        &msg_register,
        &[],
    )
    .unwrap();

    let msg_tick = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&ExecuteMsg::Tick {})?,
        funds: coins(1, NATIVE_DENOM),
    });

    let create_task_with_tick_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: Some(Boundary::Height {
                start: None,
                end: None,
            }),
            stop_on_fail: false,
            actions: vec![Action {
                msg: msg_tick.clone(),
                gas_limit: Some(250_000),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
        },
    };
    // create a task with tick
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &create_task_with_tick_msg,
        &coins(800000, NATIVE_DENOM),
    )
    .unwrap();

    let create_task_with_tick_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Once,
            boundary: Some(Boundary::Height {
                start: None,
                end: None,
            }),
            stop_on_fail: false,
            actions: vec![Action {
                msg: msg_tick,
                gas_limit: Some(250_000),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
        },
    };
    // create a second task so that another agent can be registered
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &create_task_with_tick_msg,
        &coins(600000, NATIVE_DENOM),
    )
    .unwrap();

    // might need block advancement
    app.update_block(add_little_time);

    app.execute_contract(
        Addr::unchecked(ANYONE),
        contract_addr.clone(),
        &ExecuteMsg::CheckInAgent {},
        &[],
    )
    .unwrap();

    let agents: GetAgentIdsResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetAgentIds {})
        .unwrap();
    assert_eq!(agents.active.len(), 2);
    assert!(agents.pending.is_empty());

    // block advancement, ANYONE agent didn't execute any task
    app.update_block(add_1000_blocks);

    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &ExecuteMsg::ProxyCall { task_hash: None },
            &[],
        )
        .unwrap();
    assert!(res.events.iter().any(|ev| ev.ty == "wasm"
        && ev
            .attributes
            .iter()
            .any(|attr| attr.key == "method" && attr.value == "tick")));
    assert!(res.events.iter().any(|ev| ev.ty == "wasm"
        && ev
            .attributes
            .iter()
            .any(|attr| attr.key == "account_id" && attr.value == ANYONE)));
    assert!(!res.events.iter().any(|ev| ev.ty == "wasm"
        && ev
            .attributes
            .iter()
            .any(|attr| attr.key == "account_id" && attr.value == AGENT0)));

    let agents: GetAgentIdsResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetAgentIds {})
        .unwrap();
    assert_eq!(agents.active.len(), 1);
    assert!(agents.pending.is_empty());

    Ok(())
}

#[test]
fn testing_fee_works() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let addr1 = String::from("addr1");
    let amount = coins(3, NATIVE_DENOM);
    let send = BankMsg::Send {
        to_address: addr1.clone(),
        amount: amount.clone(),
    };
    let bank_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: send.into(),
                gas_limit: None,
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
        },
    };
    let delegate = StakingMsg::Delegate {
        validator: addr1,
        amount: amount[0].clone(),
    };
    let delegate_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: delegate.into(),
                gas_limit: None,
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
        },
    };
    let total_gas = GAS_BASE_FEE + GAS_ACTION_FEE;
    let attach_per_action =
        (total_gas + (total_gas * 5 / 100)) * GAS_NUMERATOR_DEFAULT / GAS_DENOMINATOR;
    let extra = 100;
    let amount_for_three = (attach_per_action * 3) as u128 + extra;

    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &bank_msg,
        &coins(amount_for_three, NATIVE_DENOM),
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &delegate_msg,
        &coins(amount_for_three, NATIVE_DENOM),
    )
    .unwrap();

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();

    app.update_block(add_little_time);

    // let tasks: Vec<TaskResponse> = app
    //     .wrap()
    //     .query_wasm_smart(
    //         contract_addr.clone(),
    //         &QueryMsg::GetTasks {
    //             from_index: None,
    //             limit: None,
    //         },
    //     )
    //     .unwrap();
    // let tasks: Vec<(Vec<Coin>, Vec<Action>)> = tasks
    //     .into_iter()
    //     .map(|task| (task.total_deposit, task.actions))
    //     .collect();

    let proxy_call_msg = ExecuteMsg::ProxyCall { task_hash: None };
    app.execute_contract(
        Addr::unchecked(AGENT0),
        contract_addr.clone(),
        &proxy_call_msg,
        &[],
    )
    .unwrap();

    app.update_block(add_little_time);
    // let tasks: Vec<TaskResponse> = app
    //     .wrap()
    //     .query_wasm_smart(
    //         contract_addr.clone(),
    //         &QueryMsg::GetTasks {
    //             from_index: None,
    //             limit: None,
    //         },
    //     )
    //     .unwrap();
    // let tasks: Vec<(Vec<Coin>, Vec<Action>)> = tasks
    //     .into_iter()
    //     .map(|task| (task.total_deposit, task.actions))
    //     .collect();

    let proxy_call_msg = ExecuteMsg::ProxyCall { task_hash: None };
    app.execute_contract(
        Addr::unchecked(AGENT0),
        contract_addr.clone(),
        &proxy_call_msg,
        &[],
    )
    .unwrap();

    app.update_block(add_little_time);
    // let tasks: Vec<TaskResponse> = app
    //     .wrap()
    //     .query_wasm_smart(
    //         contract_addr.clone(),
    //         &QueryMsg::GetTasks {
    //             from_index: None,
    //             limit: None,
    //         },
    //     )
    //     .unwrap();
    // let tasks: Vec<(Vec<Coin>, Vec<Action>)> = tasks
    //     .into_iter()
    //     .map(|task| (task.total_deposit, task.actions))
    //     .collect();

    let proxy_call_msg = ExecuteMsg::ProxyCall { task_hash: None };
    app.execute_contract(
        Addr::unchecked(AGENT0),
        contract_addr.clone(),
        &proxy_call_msg,
        &[],
    )
    .unwrap();
}

#[test]
fn smart_query() {
    let (mut app, cw_template_contract, cw20_addr) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let cw4_id = app.store_code(cw4_template());
    let instantiate_cw4 = cw4_group::msg::InstantiateMsg {
        admin: Some(ANYONE.to_owned()),
        members: vec![
            cw4::Member {
                addr: "alice".to_string(),
                weight: 1,
            },
            cw4::Member {
                addr: "bob".to_string(),
                weight: 2,
            },
        ],
    };
    let cw4_addr = app
        .instantiate_contract(
            cw4_id,
            Addr::unchecked(ADMIN),
            &instantiate_cw4,
            &[],
            "cw4-group",
            None,
        )
        .unwrap();

    let addr1 = String::from("addr1");
    let amount = coins(3, NATIVE_DENOM);
    let send = BankMsg::Send {
        to_address: addr1,
        amount,
    };

    let head_msg = cw4_group::msg::QueryMsg::Admin {};

    let queries = SmartQueries(vec![SmartQuery {
        contract_addr: cw20_addr.to_string(),
        msg: to_binary(&cw20_base::msg::QueryMsg::Balance {
            address: "lol".to_owned(),
        })
        .unwrap(),
        path_to_msg_value: PathToValue(vec![
            ValueIndex::from("balance".to_owned()),
            ValueIndex::from("address".to_owned()),
        ]),
        path_to_query_value: PathToValue(vec![ValueIndex::from("balance".to_owned())]),
    }]);
    let smart_query = CroncatQuery::SmartQuery(SmartQueryHead {
        contract_addr: cw4_addr.to_string(),
        msg: to_binary(&head_msg).unwrap(),
        path_to_query_value: vec!["admin".to_owned().into()].into(),
        queries,
        ordering: ValueOrdering::Equal,
        value: to_binary(&Uint128::from(10_u128)).unwrap(),
    });
    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Once,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: send.clone().into(),
                gas_limit: None,
            }],
            queries: Some(vec![smart_query]),
            transforms: None,
            cw20_coins: vec![],
        },
    };

    let attached_balance = 900058;
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &create_task_msg,
        &coins(attached_balance, NATIVE_DENOM),
    )
    .unwrap();

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();

    app.update_block(add_little_time);

    app.send_tokens(
        Addr::unchecked(ADMIN),
        Addr::unchecked("addr2"),
        &coins(1, NATIVE_DENOM),
    )
    .unwrap();

    let tasks_with_queries: Vec<TaskWithQueriesResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetTasksWithQueries {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    let our_task = tasks_with_queries.first();
    assert!(our_task.is_some());
    let task_hash = our_task.unwrap().task_hash.as_ref();

    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &ExecuteMsg::ProxyCall {
                task_hash: Some(String::from(task_hash)),
            },
            &[],
        )
        .unwrap();

    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "task_hash" && attr.value == task_hash)));
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "method" && attr.value == "proxy_callback")));

    let tasks_with_queries: Vec<TaskWithQueriesResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetTasksWithQueries {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert!(tasks_with_queries.is_empty());
}

#[test]
fn insertable_query_res_positive() {
    let (mut app, cw_template_contract, cw20_addr) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let cw4_id = app.store_code(cw4_template());
    let instantiate_cw4 = cw4_group::msg::InstantiateMsg {
        admin: Some(ADMIN.to_owned()),
        members: vec![
            cw4::Member {
                addr: "alice".to_string(),
                weight: 1,
            },
            cw4::Member {
                addr: "bob".to_string(),
                weight: 2,
            },
        ],
    };
    let cw4_addr = app
        .instantiate_contract(
            cw4_id,
            Addr::unchecked(ADMIN),
            &instantiate_cw4,
            &[],
            "cw4-group",
            None,
        )
        .unwrap();

    // Send cw20 coins you plan to use
    app.execute_contract(
        Addr::unchecked(ANYONE),
        cw20_addr.clone(),
        &cw20_base::msg::ExecuteMsg::Send {
            contract: contract_addr.to_string(),
            amount: 10u128.into(),
            msg: vec![].into(),
        },
        &[],
    )
    .unwrap();

    let cw20_send = to_binary(&cw20_base::msg::ExecuteMsg::Transfer {
        recipient: "lol".to_owned(),
        amount: Uint128::new(5),
    })
    .unwrap();
    let query = CroncatQuery::GenericQuery(GenericQuery {
        contract_addr: cw4_addr.to_string(),
        msg: to_binary(&cw4_group::msg::QueryMsg::Admin {}).unwrap(),
        path_to_value: vec!["admin".to_owned().into()].into(),
        ordering: ValueOrdering::NotEqual,
        value: to_binary(&ADMIN.to_owned()).unwrap(),
    });
    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Once,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: WasmMsg::Execute {
                    contract_addr: cw20_addr.to_string(),
                    msg: cw20_send,
                    funds: vec![],
                }
                .into(),
                gas_limit: Some(300_000),
            }],
            queries: Some(vec![query]),
            transforms: Some(vec![Transform {
                action_idx: 0,
                query_idx: 0,
                action_path: PathToValue(vec![
                    ValueIndex::from("transfer".to_string()),
                    ValueIndex::from("recipient".to_string()),
                ]),
                query_response_path: PathToValue(vec![]),
            }]),
            cw20_coins: vec![Cw20Coin {
                address: cw20_addr.to_string(),
                amount: 10u128.into(),
            }],
        },
    };

    let attached_balance = 900058;
    app.execute_contract(
        Addr::unchecked(ANYONE),
        contract_addr.clone(),
        &create_task_msg,
        &coins(attached_balance, NATIVE_DENOM),
    )
    .unwrap();

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();

    app.update_block(add_little_time);

    let tasks_with_queries: Vec<TaskWithQueriesResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetTasksWithQueries {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    let our_task = tasks_with_queries.first();
    assert!(our_task.is_some());
    let task_hash: &str = our_task.unwrap().task_hash.as_ref();

    let res: ContractError = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &ExecuteMsg::ProxyCall {
                task_hash: Some(String::from(task_hash)),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(res, ContractError::QueriesNotReady { index: 0 });

    let old_balance_of_agent3: cw20::BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            cw20_addr.clone(),
            &cw20_base::msg::QueryMsg::Balance {
                address: AGENT3.to_owned(),
            },
        )
        .unwrap();

    // Replace admin
    app.execute_contract(
        Addr::unchecked(ADMIN),
        cw4_addr.clone(),
        &cw4_group::msg::ExecuteMsg::UpdateAdmin {
            admin: Some(AGENT3.to_owned()),
        },
        &[],
    )
    .unwrap();

    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &ExecuteMsg::ProxyCall {
                task_hash: Some(String::from(task_hash)),
            },
            &[],
        )
        .unwrap();
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "task_hash" && attr.value == task_hash)));
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "method" && attr.value == "proxy_callback")));

    let tasks_with_queries: Vec<TaskWithQueriesResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetTasksWithQueries {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert!(tasks_with_queries.is_empty());

    let new_balance_of_agent3: cw20::BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            cw20_addr.clone(),
            &cw20_base::msg::QueryMsg::Balance {
                address: AGENT3.to_owned(),
            },
        )
        .unwrap();
    assert_eq!(
        new_balance_of_agent3.balance - old_balance_of_agent3.balance,
        Uint128::from(5u128)
    );
}

#[ignore = "it gets cancelled too early now, have to redo this test"]
#[test]
fn insertable_query_res_negative() {
    let (mut app, cw_template_contract, cw20_addr) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let cw4_id = app.store_code(cw4_template());
    let instantiate_cw4 = cw4_group::msg::InstantiateMsg {
        admin: Some(ADMIN.to_owned()),
        members: vec![
            cw4::Member {
                addr: "alice".to_string(),
                weight: 1,
            },
            cw4::Member {
                addr: "bob".to_string(),
                weight: 2,
            },
        ],
    };
    let cw4_addr = app
        .instantiate_contract(
            cw4_id,
            Addr::unchecked(ADMIN),
            &instantiate_cw4,
            &[],
            "cw4-group",
            None,
        )
        .unwrap();

    // Send cw20 coins you plan to use
    app.execute_contract(
        Addr::unchecked(ANYONE),
        cw20_addr.clone(),
        &cw20_base::msg::ExecuteMsg::Send {
            contract: contract_addr.to_string(),
            amount: 10u128.into(),
            msg: vec![].into(),
        },
        &[],
    )
    .unwrap();

    let cw20_send = to_binary(&cw20_base::msg::ExecuteMsg::Transfer {
        recipient: "lol".to_owned(),
        amount: Uint128::new(5),
    })
    .unwrap();
    let query = CroncatQuery::GenericQuery(GenericQuery {
        contract_addr: cw4_addr.to_string(),
        msg: to_binary(&cw4_group::msg::QueryMsg::Admin {}).unwrap(),
        path_to_value: vec!["admin".to_owned().into()].into(),
        ordering: ValueOrdering::NotEqual,
        value: to_binary(&ADMIN.to_owned()).unwrap(),
    });
    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Once,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: WasmMsg::Execute {
                    contract_addr: cw20_addr.to_string(),
                    msg: cw20_send,
                    funds: vec![],
                }
                .into(),
                gas_limit: None,
            }],
            queries: Some(vec![query]),
            transforms: Some(vec![Transform {
                action_idx: 0,
                query_idx: 0,
                action_path: PathToValue(vec![
                    ValueIndex::from("transfer".to_string()),
                    ValueIndex::from("recipient".to_string()),
                ]),
                query_response_path: PathToValue(vec![]),
            }]),
            cw20_coins: vec![Cw20Coin {
                address: cw20_addr.to_string(),
                // Notice that would be not enough
                amount: 1u128.into(),
            }],
        },
    };

    let attached_balance = 900058;
    app.execute_contract(
        Addr::unchecked(ANYONE),
        contract_addr.clone(),
        &create_task_msg,
        &coins(attached_balance, NATIVE_DENOM),
    )
    .unwrap();

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();

    app.update_block(add_little_time);

    let tasks_with_queries: Vec<TaskWithQueriesResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetTasksWithQueries {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    let our_task = tasks_with_queries.first();
    assert!(our_task.is_some());
    let task_hash: &str = our_task.unwrap().task_hash.as_ref();

    let res: ContractError = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &ExecuteMsg::ProxyCall {
                task_hash: Some(String::from(task_hash)),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(res, ContractError::QueriesNotReady { index: 0 });

    let old_balance_of_agent3: cw20::BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            cw20_addr.clone(),
            &cw20_base::msg::QueryMsg::Balance {
                address: AGENT3.to_owned(),
            },
        )
        .unwrap();

    // Replace admin
    app.execute_contract(
        Addr::unchecked(ADMIN),
        cw4_addr.clone(),
        &cw4_group::msg::ExecuteMsg::UpdateAdmin {
            admin: Some(AGENT3.to_owned()),
        },
        &[],
    )
    .unwrap();

    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &ExecuteMsg::ProxyCall {
                task_hash: Some(String::from(task_hash)),
            },
            &[],
        )
        .unwrap();
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "task_hash" && attr.value == task_hash)));
    assert!(res.events.iter().any(|ev| ev
        .attributes
        .iter()
        .any(|attr| attr.key == "task_removed_without_execution")));

    let tasks_with_queries: Vec<TaskWithQueriesResponse> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetTasksWithQueries {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert!(tasks_with_queries.is_empty());

    let new_balance_of_agent3: cw20::BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            cw20_addr.clone(),
            &cw20_base::msg::QueryMsg::Balance {
                address: AGENT3.to_owned(),
            },
        )
        .unwrap();
    assert_eq!(
        new_balance_of_agent3.balance - old_balance_of_agent3.balance,
        Uint128::from(0u128)
    );
}

#[test]
fn test_error_in_reply() {
    let (mut app, cw_template_contract, _cw20_addr, governance_addr) =
        proper_instantiate_with_dao(None, None, None, None);
    let contract_addr = cw_template_contract.addr();

    // quick agent register
    let msg = ExecuteMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
        .unwrap();

    app.update_block(add_little_time);

    let governance_modules: Vec<ProposalModule> = app
        .wrap()
        .query_wasm_smart(
            governance_addr.clone(),
            &dao_core::msg::QueryMsg::ProposalModules {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    let govmod_single = governance_modules.into_iter().next().unwrap().address;

    let govmod_config: dao_proposal_single::state::Config = app
        .wrap()
        .query_wasm_smart(
            govmod_single.clone(),
            &dao_proposal_single::msg::QueryMsg::Config {},
        )
        .unwrap();
    let dao = govmod_config.dao;
    let voting_module: Addr = app
        .wrap()
        .query_wasm_smart(dao, &dao_core::msg::QueryMsg::VotingModule {})
        .unwrap();
    let staking_contract: Addr = app
        .wrap()
        .query_wasm_smart(
            voting_module.clone(),
            &dao_voting_cw20_staked::msg::QueryMsg::StakingContract {},
        )
        .unwrap();
    let token_contract: Addr = app
        .wrap()
        .query_wasm_smart(
            voting_module,
            &dao_interface::voting::Query::TokenContract {},
        )
        .unwrap();

    // Stake some tokens so we can propose
    let msg = cw20::Cw20ExecuteMsg::Send {
        contract: staking_contract.to_string(),
        amount: Uint128::new(2000),
        msg: to_binary(&cw20_stake::msg::ReceiveMsg::Stake {}).unwrap(),
    };
    app.execute_contract(Addr::unchecked(ADMIN), token_contract.clone(), &msg, &[])
        .unwrap();
    app.update_block(add_little_time);

    app.execute_contract(
        Addr::unchecked(ADMIN),
        govmod_single.clone(),
        &dao_proposal_single::msg::ExecuteMsg::Propose(SingleChoiceProposeMsg {
            title: "Cron".to_string(),
            description: "Cat".to_string(),
            msgs: vec![],
            proposer: None,
        }),
        &[],
    )
    .unwrap();

    let execute_msg = dao_proposal_single::msg::ExecuteMsg::Execute { proposal_id: 1 };

    // create a task for executing proposal
    let wasm = WasmMsg::Execute {
        contract_addr: governance_addr.to_string(),
        msg: to_binary(&execute_msg).unwrap(),
        funds: vec![],
    };
    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Once,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: wasm.clone().into(),
                gas_limit: Some(200_000),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
        },
    };

    let attached_balance = 58333;
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &create_task_msg,
        &coins(attached_balance, NATIVE_DENOM),
    )
    .unwrap();
    app.update_block(add_little_time);

    // execute proxy_call
    let proxy_call_msg = ExecuteMsg::ProxyCall { task_hash: None };
    let res = app
        .execute_contract(
            Addr::unchecked(AGENT0),
            contract_addr.clone(),
            &proxy_call_msg,
            &vec![],
        )
        .unwrap();

    // Check attributes, should have an error since we can't execute proposal yet
    let mut without_failure: bool = false;
    for e in res.events {
        for a in e.attributes {
            if a.key == "with_failure" && a.value.contains("error executing WasmMsg") {
                without_failure = true;
            }
        }
    }
    assert!(without_failure);
}

#[test]
fn empty_actions_not_allowed() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let empty_actions = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Once,
            boundary: None,
            stop_on_fail: false,
            actions: vec![],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
        },
    };

    let total_gas = GAS_BASE_FEE + GAS_ACTION_FEE;
    let attach_per_action = (total_gas + (total_gas * 5 / 100)) / GAS_NUMERATOR_DEFAULT;
    let amount_for_three = (attach_per_action) as u128;

    let res: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &empty_actions,
            &coins(amount_for_three, NATIVE_DENOM),
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(res, ContractError::CoreError(CoreError::InvalidAction {}));
}
