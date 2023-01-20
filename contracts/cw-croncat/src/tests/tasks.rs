use super::helpers::{ADMIN, ANYONE, NATIVE_DENOM, VERY_RICH};
use crate::contract::{
    GAS_ACTION_FEE, GAS_ADJUSTMENT_NUMERATOR_DEFAULT, GAS_BASE_FEE, GAS_DENOMINATOR,
    GAS_NUMERATOR_DEFAULT, GAS_QUERY_FEE, GAS_WASM_QUERY_FEE,
};
use crate::tests::helpers::proper_instantiate;
use crate::ContractError;
use cosmwasm_std::{
    coin, coins, to_binary, Addr, BankMsg, CosmosMsg, StakingMsg, StdResult, Timestamp, Uint128,
    Uint64, WasmMsg,
};
use cw2::ContractVersion;
use cw20::Balance;
use cw_croncat_core::error::CoreError;
use cw_croncat_core::msg::{
    ExecuteMsg, GetBalancesResponse, GetSlotHashesResponse, GetSlotIdsResponse, QueryMsg,
    SimulateTaskResponse, TaskRequest, TaskResponse, TaskWithQueriesResponse,
};
use cw_croncat_core::types::{Action, Boundary, CheckedBoundary, GenericBalance, Interval, Task};
use cw_multi_test::Executor;
use cw_rules_core::types::{CheckOwnerOfNft, CroncatQuery, HasBalanceGte};
use cw_utils::NativeBalance;
use hex::ToHex;
use sha2::{Digest, Sha256};
use std::convert::TryInto;

#[test]
fn query_task_hash_success() {
    let (app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let to_address = String::from("you");
    let amount = coins(1015, "earth");
    let bank = BankMsg::Send { to_address, amount };
    let msg: CosmosMsg = bank.clone().into();
    let version = ContractVersion {
        version: "0.0.1".to_string(),
        contract: "nobidy".to_string(),
    };

    let task = Task {
        owner_id: Addr::unchecked("nobody".to_string()),
        interval: Interval::Immediate,
        boundary: CheckedBoundary {
            start: None,
            end: None,
            is_block_boundary: None,
        },
        stop_on_fail: false,
        total_deposit: GenericBalance {
            native: coins(37, NATIVE_DENOM),
            cw20: Default::default(),
        },
        amount_for_one_task: Default::default(),
        actions: vec![Action {
            msg,
            gas_limit: Some(150_000),
        }],
        queries: None,
        transforms: None,
        version: version.version,
    };

    // HASH CHECK!
    let task_hash: String = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTaskHash {
                task: Box::new(task),
            },
        )
        .unwrap();
    assert_eq!(
        "atom:05dbd09a8948de64d52e9da638b8709eb4f7cadf85a7c203c4b2889c8ae",
        task_hash
    );
}

#[test]
fn query_validate_interval_success() {
    let (app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let intervals: Vec<Interval> = vec![
        Interval::Once,
        Interval::Immediate,
        Interval::Block(12345),
        Interval::Cron("0 0 * * * *".to_string()),
    ];
    for i in intervals.iter() {
        let valid: bool = app
            .wrap()
            .query_wasm_smart(
                &contract_addr.clone(),
                &QueryMsg::ValidateInterval {
                    interval: i.to_owned(),
                },
            )
            .unwrap();
        assert!(valid);
    }
}

#[test]
fn query_get_tasks() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let validator = String::from("you");
    let amount = coin(3, NATIVE_DENOM);
    let stake = StakingMsg::Delegate { validator, amount };
    let msg: CosmosMsg = stake.clone().into();

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg,
                gas_limit: Some(150_000),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
            sender: None,
        },
    };

    // create a task
    app.execute_contract(
        Addr::unchecked(ANYONE),
        contract_addr.clone(),
        &create_task_msg,
        &coins(315006, NATIVE_DENOM),
    )
    .unwrap();

    // check storage has the task
    let all_tasks: Vec<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTasks {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(all_tasks.len(), 1);

    let owner_tasks: Vec<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTasksByOwner {
                owner_id: ANYONE.to_string(),
            },
        )
        .unwrap();
    assert_eq!(owner_tasks.len(), 1);
}

#[test]
fn query_simulate_task_estimated_gas() {
    let (app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let send = BankMsg::Send {
        to_address: String::from(ANYONE),
        amount: vec![coin(1, NATIVE_DENOM)],
    };
    let send2 = BankMsg::Send {
        to_address: String::from(ANYONE),
        amount: vec![coin(2, NATIVE_DENOM)],
    };
    let msg: CosmosMsg = send.clone().into();
    let msg2: CosmosMsg = send2.clone().into();

    // One action
    let task_request = TaskRequest {
        interval: Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };

    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    assert_eq!(simulate.estimated_gas, GAS_BASE_FEE + GAS_ACTION_FEE);

    // One action with gas_limit
    let task_request = TaskRequest {
        interval: Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: Some(123_321),
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };

    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    assert_eq!(simulate.estimated_gas, GAS_BASE_FEE + 123_321);

    // Two actions
    let task_request = TaskRequest {
        interval: Interval::Immediate,
        boundary: Some(Boundary::Height {
            start: None,
            end: Some(Uint64::from(12346u64)),
        }),
        stop_on_fail: false,
        actions: vec![
            Action {
                msg: msg.clone(),
                gas_limit: None,
            },
            Action {
                msg: msg2.clone(),
                gas_limit: None,
            },
        ],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };

    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    assert_eq!(simulate.estimated_gas, GAS_BASE_FEE + 2 * GAS_ACTION_FEE);

    // Two actions, one of them with gas_limit
    let task_request = TaskRequest {
        interval: Interval::Immediate,
        boundary: Some(Boundary::Height {
            start: None,
            end: Some(Uint64::from(12346u64)),
        }),
        stop_on_fail: false,
        actions: vec![
            Action {
                msg: msg.clone(),
                gas_limit: None,
            },
            Action {
                msg: msg2.clone(),
                gas_limit: Some(123_321),
            },
        ],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };

    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    assert_eq!(
        simulate.estimated_gas,
        GAS_BASE_FEE + GAS_ACTION_FEE + 123_321
    );

    // Two actions, both with gas_limit
    let task_request = TaskRequest {
        interval: Interval::Immediate,
        boundary: Some(Boundary::Height {
            start: None,
            end: Some(Uint64::from(12346u64)),
        }),
        stop_on_fail: false,
        actions: vec![
            Action {
                msg: msg.clone(),
                gas_limit: Some(321_123),
            },
            Action {
                msg: msg2.clone(),
                gas_limit: Some(123_321),
            },
        ],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };

    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    assert_eq!(simulate.estimated_gas, GAS_BASE_FEE + 321_123 + 123_321);

    // One action and query CheckOwnerOfNft
    // Estimated gas increases by 2 * GAS_WASM_QUERY_FEE
    let task_request = TaskRequest {
        interval: Interval::Block(10),
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: Some(vec![CroncatQuery::CheckOwnerOfNft(CheckOwnerOfNft {
            address: ADMIN.to_string(),
            nft_address: "NFT".to_string(),
            token_id: "TOKEN".to_string(),
        })]),
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };

    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    assert_eq!(
        simulate.estimated_gas,
        GAS_BASE_FEE + GAS_ACTION_FEE + 2 * GAS_WASM_QUERY_FEE,
    );

    // One action and query HasBalanceGte
    // This query adds to estimated gas GAS_WASM_QUERY_FEE and GAS_QUERY_FEE
    let task_request = TaskRequest {
        interval: Interval::Cron("1 * * * * *".to_string()),
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: Some(vec![CroncatQuery::HasBalanceGte(HasBalanceGte {
            address: ADMIN.to_string(),
            required_balance: Balance::Native(NativeBalance(vec![coin(100, NATIVE_DENOM)])),
        })]),
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };

    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    assert_eq!(
        simulate.estimated_gas,
        GAS_BASE_FEE + GAS_ACTION_FEE + GAS_WASM_QUERY_FEE + GAS_QUERY_FEE,
    );
}

#[test]
fn query_simulate_task_hash() {
    let (app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let send = BankMsg::Send {
        to_address: String::from(ANYONE),
        amount: vec![coin(1, NATIVE_DENOM)],
    };
    let send2 = BankMsg::Send {
        to_address: String::from(ANYONE),
        amount: vec![coin(2, NATIVE_DENOM)],
    };
    let msg: CosmosMsg = send.clone().into();
    let msg2: CosmosMsg = send2.clone().into();

    let task_request = TaskRequest {
        interval: Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![
            Action {
                msg,
                gas_limit: Some(150_000),
            },
            Action {
                msg: msg2.clone(),
                gas_limit: None,
            },
        ],
        queries: Some(vec![CroncatQuery::CheckOwnerOfNft(CheckOwnerOfNft {
            address: ADMIN.to_string(),
            nft_address: "NFT".to_string(),
            token_id: "TOKEN".to_string(),
        })]),
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };

    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    let message = format!(
        "{:?}{:?}{:?}{:?}{:?}{:?}",
        Addr::unchecked(ANYONE),
        task_request.interval,
        CheckedBoundary::new(task_request.boundary, &task_request.interval).unwrap(),
        task_request.actions,
        task_request.queries,
        task_request.transforms
    );
    let hash = Sha256::digest(message.as_bytes());
    let encoded: String = hash.encode_hex();
    let prefix = NATIVE_DENOM;

    let (_, l) = encoded.split_at(prefix.len() + 1);
    let task_hash = format!("{}:{}", prefix, l);

    assert_eq!(simulate.task_hash, task_hash);
}

#[test]
fn query_simulate_task_occurrences_with_queries() {
    let (app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let send = BankMsg::Send {
        to_address: String::from(ANYONE),
        amount: vec![coin(1, NATIVE_DENOM)],
    };
    let msg: CosmosMsg = send.clone().into();

    // Interval::Once with query
    let task_request = TaskRequest {
        interval: Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: Some(vec![CroncatQuery::CheckOwnerOfNft(CheckOwnerOfNft {
            address: ADMIN.to_string(),
            nft_address: "NFT".to_string(),
            token_id: "TOKEN".to_string(),
        })]),
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(352820, NATIVE_DENOM)),
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 1);

    // Interval::Immediate with query
    let task_request = TaskRequest {
        interval: Interval::Immediate,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: Some(vec![CroncatQuery::CheckOwnerOfNft(CheckOwnerOfNft {
            address: ADMIN.to_string(),
            nft_address: "NFT".to_string(),
            token_id: "TOKEN".to_string(),
        })]),
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(352820, NATIVE_DENOM)),
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 1);

    // Interval::Block with query
    let task_request = TaskRequest {
        interval: Interval::Block(15),
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: Some(vec![CroncatQuery::CheckOwnerOfNft(CheckOwnerOfNft {
            address: ADMIN.to_string(),
            nft_address: "NFT".to_string(),
            token_id: "TOKEN".to_string(),
        })]),
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(352820, NATIVE_DENOM)),
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 1);

    // Interval::Cron with query
    let task_request = TaskRequest {
        interval: Interval::Cron("* 1 * * * *".to_string()),
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: Some(vec![CroncatQuery::CheckOwnerOfNft(CheckOwnerOfNft {
            address: ADMIN.to_string(),
            nft_address: "NFT".to_string(),
            token_id: "TOKEN".to_string(),
        })]),
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(352820, NATIVE_DENOM)),
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 1);
}

#[test]
fn query_simulate_task_occurrences_once() {
    let (app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let send = BankMsg::Send {
        to_address: String::from(ANYONE),
        amount: vec![coin(1, NATIVE_DENOM)],
    };
    let msg: CosmosMsg = send.clone().into();

    let task_request = TaskRequest {
        interval: Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };

    // With funds
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(352820, NATIVE_DENOM)),
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 1);

    // Without funds
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: None,
            },
        )
        .unwrap();

    assert_eq!(simulate.occurrences, 1);
}

#[test]
fn query_simulate_task_occurrences_immediate() {
    let (app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let send = BankMsg::Send {
        to_address: String::from(ANYONE),
        amount: vec![coin(1, NATIVE_DENOM)],
    };
    let msg: CosmosMsg = send.clone().into();

    let task_request = TaskRequest {
        interval: Interval::Immediate,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };

    // With funds
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(352820, NATIVE_DENOM)),
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 1);

    // Without funds
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: None,
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 1);
}

#[test]
fn query_simulate_task_occurrences_block_with_funds() {
    let (app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let send = BankMsg::Send {
        to_address: String::from(ANYONE),
        amount: vec![coin(1, NATIVE_DENOM)],
    };
    let send2 = BankMsg::Send {
        to_address: String::from(ANYONE),
        amount: vec![coin(2, NATIVE_DENOM)],
    };
    let msg: CosmosMsg = send.clone().into();
    let msg2: CosmosMsg = send2.clone().into();

    // Interval::Block with one action, no boundaries
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    assert_eq!(simulate.occurrences, 36);

    // Interval::Block with two actions, no boundaries
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: None,
        stop_on_fail: false,
        actions: vec![
            Action {
                msg: msg.clone(),
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
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 35283
    // Thus 1_000_000 should be enough for 28 proxy_call's
    assert_eq!(simulate.occurrences, 28);

    // Interval::Block with one action
    // the start boundary is less than the current block
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: Some(Boundary::Height {
            // Start is less than the current block
            start: Some(12344u64.into()),
            end: None,
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // Start boundary doesn't impact it
    assert_eq!(simulate.occurrences, 36);

    // Interval::Block with one action
    // the start boundary is bigger than the current block number
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: Some(Boundary::Height {
            // Start is bigger than the current block number
            start: Some(12350u64.into()),
            end: None,
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // Start boundary doesn't impact it
    assert_eq!(simulate.occurrences, 36);

    // Interval::Block with one action
    // End boundary is too big to impact occurrences
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: Some(Boundary::Height {
            start: None,
            // 12345 is the current block
            // The task will be scheduled for blocks 12350, 12355, 12360 ... 12520, 12525
            end: Some((12345 + 5 * 36 as u64).into()),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // End boundary doesn't impact it
    assert_eq!(simulate.occurrences, 36);

    // Interval::Block with one action
    // End boundary limits the occurrences
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: Some(Boundary::Height {
            start: None,
            // 12345 is the current block
            // The task will be scheduled for blocks 12350, 12355, 12360 ... 12520
            // 12525 is out of boundary
            end: Some((12345 + 5 * 36 - 1 as u64).into()),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // But the end boundary doesn't allow the last proxy_call
    assert_eq!(simulate.occurrences, 35);

    // Interval::Block with one action
    // End boundary is big, thus occurrences are calculated by funds
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: Some(Boundary::Height {
            start: None,
            // 12345 is the current block
            // The task will be scheduled for blocks 12350, 12355, 12360 ... 12520
            end: Some((12345 + 5 * 100 as u64).into()),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    assert_eq!(simulate.occurrences, 36);

    // Interval::Block with one action
    // Start boundary is smaller than the current block number
    // End boundary limits the occurrences
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: Some(Boundary::Height {
            start: Some(12000u64.into()),
            end: Some((12345 + 5 * 36 - 1 as u64).into()),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // But the end boundary doesn't allow the last proxy_call
    // Start boundary doesn't impact calculations
    assert_eq!(simulate.occurrences, 35);

    // Start boundary is smaller than the current block number
    // End boundary doesn't limit the occurrences
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: Some(Boundary::Height {
            start: Some(12000u64.into()),
            end: Some((12345 + 5 * 36 as u64).into()),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // The end boundary doesn't limit the number of proxy_call's
    // Start boundary doesn't impact calculations
    assert_eq!(simulate.occurrences, 36);

    // Start boundary is smaller than the current block number
    // End boundary is big, thus occurrences are calculated by funds
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: Some(Boundary::Height {
            start: Some(12000u64.into()),
            end: Some((12345 + 5 * 100 as u64).into()),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // Start boundary doesn't impact calculations
    assert_eq!(simulate.occurrences, 36);

    // Start boundary is some future block
    // End boundary doesn't limit the occurrences
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: Some(Boundary::Height {
            start: Some(15000u64.into()),
            // 14995 here is because the start block is in the future
            // and the task will be scheduled for block 15000
            end: Some((14995 + 5 * 36 as u64).into()),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // The difference between end and start is bid enough for all 36 proxy_call's
    assert_eq!(simulate.occurrences, 36);

    // Start boundary is some future block
    // End boundary limits the occurrences
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: Some(Boundary::Height {
            start: Some(15000u64.into()),
            // 14995 here is because the start block is in the future
            // and the task will be scheduled for block 15000
            end: Some((14995 + 5 * 36 - 1 as u64).into()),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // The difference between end and start allows only 35 proxy_call's
    assert_eq!(simulate.occurrences, 35);

    // Start boundary is some future block
    // End boundary is big, thus occurrences are calculated by funds
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: Some(Boundary::Height {
            start: Some(15000u64.into()),
            end: Some((14995 + 5 * 100 as u64).into()),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    assert_eq!(simulate.occurrences, 36);
}

#[test]
fn query_simulate_task_occurrences_block_without_funds() {
    let (app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let send = BankMsg::Send {
        to_address: String::from(ANYONE),
        amount: vec![coin(1, NATIVE_DENOM)],
    };
    let msg: CosmosMsg = send.clone().into();

    // Test cases when funds aren't provided

    // Interval::Block with one action
    // No funds and boundaries
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: None,
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 1);

    // No funds, boundary has only start in the past
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: Some(Boundary::Height {
            start: Some(Uint64::from(12360u64)),
            end: None,
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: None,
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 1);

    // No funds, boundary has only start in the future
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: Some(Boundary::Height {
            start: Some(Uint64::from(12360u64)),
            end: None,
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: None,
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 1);

    // No funds, boundary has only end
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: Some(Boundary::Height {
            start: None,
            // The task is scheduled for 12350, 12355, ... 12845
            // 100 occurrences total
            end: Some(Uint64::from(12345 + 5 * 100 as u64)),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: None,
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 100);

    // No funds
    // Start block is smaller that the current block
    // Boundary has end
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: Some(Boundary::Height {
            start: Some(Uint64::from(12300u64)),
            // The task is scheduled for 12350, 12355, ... 12845
            // 100 occurrences total
            end: Some(Uint64::from(12345 + 5 * 100 as u64)),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: None,
            },
        )
        .unwrap();
    // Start doesn't impact occurrences
    assert_eq!(simulate.occurrences, 100);

    // No funds
    // Start block is bigger that the current block
    // Boundary has end
    let task_request = TaskRequest {
        interval: Interval::Block(5),
        boundary: Some(Boundary::Height {
            start: Some(Uint64::from(15000u64)),
            // The task is scheduled for 15000, 15005, 15010, ... 15495
            // 100 occurrences total
            end: Some(Uint64::from(14995 + 5 * 100 as u64)),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: None,
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 100);
}

#[test]
fn query_simulate_task_occurrences_cron_with_funds() {
    let (app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();
    let current_time: u64 = 1571797419879305533;

    let send = BankMsg::Send {
        to_address: String::from(ANYONE),
        amount: vec![coin(1, NATIVE_DENOM)],
    };
    let send2 = BankMsg::Send {
        to_address: String::from(ANYONE),
        amount: vec![coin(2, NATIVE_DENOM)],
    };
    let msg: CosmosMsg = send.clone().into();
    let msg2: CosmosMsg = send2.clone().into();

    // Interval::Cron with one action, no boundaries
    // "0 * * * * *" schedules proxy_call once for a minute
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    assert_eq!(simulate.occurrences, 36);

    // Interval::Cron with two actions, no boundaries
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: None,
        stop_on_fail: false,
        actions: vec![
            Action {
                msg: msg.clone(),
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
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 35283
    // Thus 1_000_000 should be enough for 28 proxy_call's
    assert_eq!(simulate.occurrences, 28);

    // Interval::Cron with one action
    // the start boundary is less than the current time
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: Some(Boundary::Time {
            // Start is less than the current time
            start: Some(Timestamp::from_nanos(current_time).minus_seconds(100)),
            end: None,
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // Start boundary doesn't impact it
    assert_eq!(simulate.occurrences, 36);

    // Interval::Cron with one action
    // the start boundary is bigger than the current time
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: Some(Boundary::Time {
            // Start is bigger than the current time
            start: Some(Timestamp::from_nanos(current_time).plus_seconds(100)),
            end: None,
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // Start boundary doesn't impact it
    assert_eq!(simulate.occurrences, 36);

    // Interval::Cron with one action
    // End boundary is too big to impact occurrences
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: Some(Boundary::Time {
            start: None,
            // The task will be scheduled for 1571797440, 1571797500, ... 1571799480, 1571799510 seconds
            end: Some(Timestamp::from_nanos(current_time).plus_seconds(60 * 35)),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // End boundary doesn't impact it
    assert_eq!(simulate.occurrences, 36);

    // Interval::Cron with one action
    // End boundary limits the occurrences
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: Some(Boundary::Time {
            start: None,
            // The task will be scheduled for 1571797440, 1571797500, ... 1571799420, 1571799450 seconds
            // the last one possible proxy_call is out of boundaries
            end: Some(Timestamp::from_nanos(current_time).plus_seconds(60 * 34)),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // But the end boundary doesn't allow the last proxy_call
    assert_eq!(simulate.occurrences, 35);

    // Interval::Cron with one action
    // End boundary is big, thus occurrences are calculated by funds
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: Some(Boundary::Time {
            start: None,
            // The task will be scheduled for 1571797440, 1571797500, ... 1571799480, 1571799540 seconds
            end: Some(Timestamp::from_nanos(current_time).plus_seconds(60 * 100)),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    assert_eq!(simulate.occurrences, 36);

    // Interval::Cron with one action
    // Start boundary is smaller than the current time
    // End boundary limits the occurrences
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: Some(Boundary::Time {
            start: Some(Timestamp::from_nanos(current_time).minus_seconds(60 * 34)),
            end: Some(Timestamp::from_nanos(current_time).plus_seconds(60 * 34)),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // But the end boundary doesn't allow the last proxy_call
    // Start boundary doesn't impact calculations
    assert_eq!(simulate.occurrences, 35);

    // Start boundary is smaller than the current time
    // End boundary doesn't limit the occurrences
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: Some(Boundary::Time {
            start: Some(Timestamp::from_nanos(current_time).minus_seconds(60 * 34)),
            end: Some(Timestamp::from_nanos(current_time).plus_seconds(60 * 35)),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // The end boundary doesn't limit the number of proxy_call's
    // Start boundary doesn't impact it
    assert_eq!(simulate.occurrences, 36);

    // Start boundary is smaller than the current time
    // End boundary is big, thus occurrences are calculated by funds
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: Some(Boundary::Time {
            start: Some(Timestamp::from_nanos(current_time).minus_seconds(60 * 34)),
            end: Some(Timestamp::from_nanos(current_time).plus_seconds(60 * 100)),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // Start boundary doesn't impact calculations
    assert_eq!(simulate.occurrences, 36);

    println!("SEE HERE");
    // Start boundary is some future block
    // End boundary doesn't limit the occurrences
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: Some(Boundary::Time {
            // Schedule: 1571803440, 1571803500, ... 1571805480, 1571805510 seconds
            start: Some(Timestamp::from_nanos(current_time).plus_seconds(60 * 100)),
            end: Some(Timestamp::from_nanos(current_time).plus_seconds(60 * 135)),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // The difference between end and start is bid enough for all 36 proxy_call's
    assert_eq!(simulate.occurrences, 36);

    // Start boundary is some future time
    // End boundary limits the occurrences
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: Some(Boundary::Time {
            start: Some(Timestamp::from_nanos(current_time).plus_seconds(60 * 100)),
            end: Some(Timestamp::from_nanos(current_time).plus_seconds(60 * 134)),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    // The difference between end and start allows only 35 proxy_call's
    assert_eq!(simulate.occurrences, 35);

    // Start boundary is some future time
    // End boundary is big, thus occurrences are calculated by funds
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: Some(Boundary::Time {
            start: Some(Timestamp::from_nanos(current_time).plus_seconds(60 * 100)),
            end: Some(Timestamp::from_nanos(current_time).plus_seconds(60 * 200)),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: Some(coins(1_000_000, NATIVE_DENOM)),
            },
        )
        .unwrap();
    // For this task amount_for_one_task == 27091
    // Thus 1_000_000 should be enough for 36 proxy_call's
    assert_eq!(simulate.occurrences, 36);
}

#[test]
fn query_simulate_task_occurrences_cron_without_funds() {
    let (app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();
    let current_time = 1571797419879305533;

    let send = BankMsg::Send {
        to_address: String::from(ANYONE),
        amount: vec![coin(1, NATIVE_DENOM)],
    };
    let msg: CosmosMsg = send.clone().into();

    // Test cases when funds aren't provided

    // Interval::Cron with one action
    // No funds and boundaries
    // For this task amount_for_one_task == 27091
    // "0 * * * * *" schedules proxy_call once for a minute
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: None,
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 1);

    // No funds, boundary has only start in the past
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: Some(Boundary::Time {
            start: Some(Timestamp::from_nanos(current_time).minus_seconds(60)),
            end: None,
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: None,
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 1);

    // No funds, boundary has only start in the future
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: Some(Boundary::Time {
            start: Some(Timestamp::from_nanos(current_time).plus_seconds(100)),
            end: None,
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: None,
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 1);

    // No funds, boundary has only end
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: Some(Boundary::Time {
            start: None,
            // The task is scheduled for 1571797440, 1571797500, ... 1571797920, 1571797950 seconds
            // 10 occurrences total
            end: Some(Timestamp::from_nanos(current_time).plus_seconds(60 * 9)),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: None,
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 10);

    // No funds
    // Start block is smaller that the current block
    // Boundary has end
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: Some(Boundary::Time {
            start: Some(Timestamp::from_nanos(current_time).minus_seconds(60 * 9)),
            // The task is scheduled for 12350, 12355, ... 12845
            // 10 occurrences total
            end: Some(Timestamp::from_nanos(current_time).plus_seconds(60 * 9)),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: None,
            },
        )
        .unwrap();
    // Start doesn't impact occurrences
    assert_eq!(simulate.occurrences, 10);

    // No funds
    // Start block is bigger that the current block
    // Boundary has end
    let task_request = TaskRequest {
        interval: Interval::Cron("0 * * * * *".to_string()),
        boundary: Some(Boundary::Time {
            start: Some(Timestamp::from_nanos(current_time).plus_seconds(60)),
            end: Some(Timestamp::from_nanos(current_time).plus_seconds(60 * 10)),
        }),
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
        sender: Some(ANYONE.to_owned()),
    };
    let simulate: SimulateTaskResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::SimulateTask {
                task: task_request.clone(),
                funds: None,
            },
        )
        .unwrap();
    assert_eq!(simulate.occurrences, 10);
}

#[test]
fn query_get_tasks_pagination() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let validator = String::from("you");
    let tasks_amnt: u64 = 10;
    let from_index = 3;
    let limit = 2;
    let new_msg = |amount| ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: StakingMsg::Delegate {
                    validator: validator.clone(),
                    amount: coin(amount, NATIVE_DENOM),
                }
                .into(),
                gas_limit: Some(150_000),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
            sender: None,
        },
    };

    // create a tasks
    for amount in 1..tasks_amnt as u128 + 1 {
        app.execute_contract(
            Addr::unchecked(VERY_RICH),
            contract_addr.clone(),
            &new_msg(amount),
            &coins(315000 + 2 * amount, NATIVE_DENOM),
        )
        .unwrap();
    }
    let mut all_tasks: Vec<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTasks {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(all_tasks.len(), tasks_amnt as usize);

    // check we get right amount of tasks
    let part_of_tasks: Vec<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTasks {
                from_index: Some(from_index),
                limit: None,
            },
        )
        .unwrap();
    let expected_amnt: usize = (tasks_amnt - from_index).try_into().unwrap();
    assert_eq!(part_of_tasks.len(), expected_amnt);

    // Check it's in right order
    for i in 0..expected_amnt {
        assert_eq!(
            all_tasks[from_index as usize + i].task_hash,
            part_of_tasks[i].task_hash
        );
    }

    // and with limit
    let part_of_tasks: Vec<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTasks {
                from_index: Some(from_index),
                limit: Some(limit),
            },
        )
        .unwrap();
    let expected_amnt: usize = (limit).try_into().unwrap();
    assert_eq!(part_of_tasks.len(), expected_amnt);

    // Edge cases

    // Index out of bounds, so we return nothing
    let from_index = tasks_amnt;
    let out_of_bounds: Vec<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTasks {
                from_index: Some(from_index),
                limit: None,
            },
        )
        .unwrap();
    assert!(out_of_bounds.is_empty());

    // Returns as many elements as possible without a panic
    let from_index = tasks_amnt - 2;
    let two_last_elements: Vec<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTasks {
                from_index: Some(from_index),
                limit: Some(tasks_amnt),
            },
        )
        .unwrap();
    assert_eq!(two_last_elements.len(), 2);

    // Removed task shouldn't reorder things
    let removed_index = from_index as usize;
    app.execute_contract(
        Addr::unchecked(VERY_RICH),
        contract_addr.clone(),
        &ExecuteMsg::RemoveTask {
            task_hash: all_tasks
                .remove(removed_index) // We removed hash from original vector to match
                .task_hash,
        },
        &vec![],
    )
    .unwrap();
    let new_tasks: Vec<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTasks {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(new_tasks, all_tasks);
}

#[test]
fn check_task_create_fail_cases() -> StdResult<()> {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let validator = String::from("you");
    let amount = coin(3, NATIVE_DENOM);
    let stake = StakingMsg::Delegate { validator, amount };
    let msg: CosmosMsg = stake.clone().into();

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: msg.clone(),
                gas_limit: Some(150_000),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
            sender: None,
        },
    };
    // let task_id_str = "95c916a53fa9d26deef094f7e1ee31c00a2d47b8bf474b2e06d39aebfb1fecc7".to_string();
    // let task_id = task_id_str.clone().into_bytes();

    // Must attach funds
    let res_err = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &create_task_msg,
            &vec![],
        )
        .unwrap_err();
    assert_eq!(
        ContractError::CustomError {
            val: "Must attach funds".to_string()
        },
        res_err.downcast().unwrap()
    );

    // Create task paused
    let change_settings_msg = ExecuteMsg::UpdateSettings {
        paused: Some(true),
        owner_id: None,
        chain_name: None,
        // treasury_id: None,
        agent_fee: None,
        agents_eject_threshold: None,
        gas_price: None,
        proxy_callback_gas: None,
        slot_granularity_time: None,
        min_tasks_per_agent: None,
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
    let res_err = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &create_task_msg,
            &coins(315006, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::CustomError {
            val: "Create task paused".to_string()
        },
        res_err.downcast().unwrap()
    );
    // Set it back
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::UpdateSettings {
            paused: Some(false),
            owner_id: None,
            chain_name: None,
            // treasury_id: None,
            agent_fee: None,
            agents_eject_threshold: None,
            gas_price: None,
            proxy_callback_gas: None,
            slot_granularity_time: None,
            min_tasks_per_agent: None,
            gas_base_fee: None,
            gas_action_fee: None,
            gas_query_fee: None,
            gas_wasm_query_fee: None,
        },
        &vec![],
    )
    .unwrap();

    // Creator invalid
    let action_self = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.clone().into_string(),
        funds: vec![],
        msg: to_binary(&change_settings_msg.clone())?,
    });
    let res_err = app.execute_contract(
        Addr::unchecked(ANYONE),
        contract_addr.clone(),
        &ExecuteMsg::CreateTask {
            task: TaskRequest {
                interval: Interval::Once,
                boundary: None,
                stop_on_fail: false,
                actions: vec![Action {
                    msg: action_self.clone(),
                    gas_limit: Some(150_000),
                }],
                queries: None,
                transforms: None,
                cw20_coins: vec![],
                sender: None,
            },
        },
        &coins(13, NATIVE_DENOM),
    );
    assert_eq!(
        ContractError::CoreError(CoreError::InvalidAction {}),
        res_err.unwrap_err().downcast().unwrap()
    );

    // Must include gas_limit for WASM actions
    let action_self = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.clone().into_string(),
        funds: vec![],
        msg: to_binary(&change_settings_msg.clone())?,
    });
    let res_err = app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::CreateTask {
            task: TaskRequest {
                interval: Interval::Once,
                boundary: None,
                stop_on_fail: false,
                actions: vec![Action {
                    msg: action_self.clone(),
                    gas_limit: None,
                }],
                queries: None,
                transforms: None,
                cw20_coins: vec![],
                sender: None,
            },
        },
        &coins(13, NATIVE_DENOM),
    );
    assert_eq!(
        ContractError::CoreError(CoreError::NoGasLimit {}),
        res_err.unwrap_err().downcast().unwrap()
    );

    // Interval invalid
    let res_err = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &ExecuteMsg::CreateTask {
                task: TaskRequest {
                    interval: Interval::Cron("faux_paw".to_string()),
                    boundary: None,
                    stop_on_fail: false,
                    actions: vec![Action {
                        msg: msg.clone(),
                        gas_limit: Some(150_000),
                    }],
                    queries: None,
                    transforms: None,
                    cw20_coins: vec![],
                    sender: None,
                },
            },
            &coins(13, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::CustomError {
            val: "Interval invalid".to_string()
        },
        res_err.downcast().unwrap()
    );

    // Task already exists
    app.execute_contract(
        Addr::unchecked(ANYONE),
        contract_addr.clone(),
        &create_task_msg,
        &coins(315006, NATIVE_DENOM),
    )
    .unwrap();
    let res_err = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &create_task_msg,
            &coins(315006, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::CustomError {
            val: "Task already exists".to_string()
        },
        res_err.downcast().unwrap()
    );

    // Task ended
    let res_err = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &ExecuteMsg::CreateTask {
                task: TaskRequest {
                    interval: Interval::Block(12346),
                    boundary: Some(Boundary::Height {
                        start: None,
                        end: Some(1u64.into()),
                    }),
                    stop_on_fail: false,
                    actions: vec![Action {
                        msg,
                        gas_limit: Some(150_000),
                    }],
                    queries: None,
                    transforms: None,
                    cw20_coins: vec![],
                    sender: None,
                },
            },
            &coins(315006, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::CustomError {
            val: "Task ended".to_string()
        },
        res_err.downcast().unwrap()
    );

    // TODO: (needs impl!) Not enough task balance to execute job

    Ok(())
}

#[test]
fn check_task_create_success() -> StdResult<()> {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let validator = String::from("you");
    let amount = coin(3, NATIVE_DENOM);
    let stake = StakingMsg::Delegate { validator, amount };
    let msg: CosmosMsg = stake.clone().into();

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg,
                gas_limit: Some(150_000),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
            sender: None,
        },
    };

    // create a task
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
    let mut task_hash = String::new();
    for e in res.events {
        for a in e.attributes {
            if a.key == "task_hash" && a.value.len() > 0 {
                has_created_hash = true;
                task_hash = a.value;
            }
        }
    }
    assert!(has_created_hash);

    // check storage has the task
    let new_task: Option<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTask {
                task_hash: task_hash.clone(),
            },
        )
        .unwrap();
    assert!(new_task.is_some());
    println!("new_task {:?} {:?}", new_task, task_hash.clone());
    if let Some(t) = new_task {
        assert_eq!(Addr::unchecked(ANYONE), t.owner_id);
        assert_eq!(Interval::Immediate, t.interval);
        assert!(t.boundary.is_some());
        assert_eq!(false, t.stop_on_fail);
        assert_eq!(coins(315006, NATIVE_DENOM), t.total_deposit);
        assert_eq!(task_hash.clone(), t.task_hash);
    }

    // get slot ids
    let slot_ids: GetSlotIdsResponse = app
        .wrap()
        .query_wasm_smart(&contract_addr.clone(), &QueryMsg::GetSlotIds {})
        .unwrap();
    let s_1: Vec<u64> = Vec::new();
    assert_eq!(s_1, slot_ids.time_ids);
    assert_eq!(vec![12346], slot_ids.block_ids);

    // get slot hashs
    let slot_info: GetSlotHashesResponse = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetSlotHashes { slot: None },
        )
        .unwrap();
    let s_3: Vec<String> = Vec::new();
    assert_eq!(12346, slot_info.block_id);
    assert_eq!(vec![task_hash], slot_info.block_task_hash);
    assert_eq!(0, slot_info.time_id);
    assert_eq!(s_3, slot_info.time_task_hash);

    Ok(())
}

#[test]
fn check_task_with_queries_create_success() -> StdResult<()> {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let validator = String::from("you");
    let amount = coin(3, NATIVE_DENOM);
    let stake = StakingMsg::Delegate { validator, amount };
    let msg: CosmosMsg = stake.clone().into();

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg,
                gas_limit: Some(150_000),
            }],
            queries: Some(vec![CroncatQuery::HasBalanceGte(HasBalanceGte {
                address: "foo".to_string(),
                required_balance: coins(5, "bar").into(),
            })]),
            transforms: None,
            cw20_coins: vec![],
            sender: None,
        },
    };

    // create a task
    let res = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &create_task_msg,
            &coins(315006, NATIVE_DENOM),
        )
        .unwrap();

    let tasks_with_queries: Vec<TaskWithQueriesResponse> = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTasksWithQueries {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();

    let tasks: Vec<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTasks {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();

    assert_eq!(tasks_with_queries.len(), 1);
    assert_eq!(tasks.len(), 0);

    let mut has_created_hash: bool = false;
    for e in res.events {
        for a in e.attributes {
            if a.key == "with_queries" && a.value == "true" {
                has_created_hash = true;
            }
        }
    }
    assert!(has_created_hash);
    Ok(())
}

#[test]
fn check_task_with_queries_and_without_create_success() -> StdResult<()> {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let validator = String::from("you");
    let amount = coin(3, NATIVE_DENOM);
    let stake = StakingMsg::Delegate { validator, amount };
    let msg: CosmosMsg = stake.clone().into();

    let with_queries_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: msg.clone(),
                gas_limit: Some(150_000),
            }],
            queries: Some(vec![CroncatQuery::HasBalanceGte(HasBalanceGte {
                address: "foo".to_string(),
                required_balance: coins(5, "bar").into(),
            })]),
            transforms: None,
            cw20_coins: vec![],
            sender: None,
        },
    };

    let without_queries_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg,
                gas_limit: Some(150_000),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
            sender: None,
        },
    };

    // create a task
    let res = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &with_queries_msg,
            &coins(315006, NATIVE_DENOM),
        )
        .unwrap();

    let res2 = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &without_queries_msg,
            &coins(315006, NATIVE_DENOM),
        )
        .unwrap();

    let tasks_with_queries: Vec<TaskWithQueriesResponse> = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTasksWithQueries {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();

    let tasks: Vec<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTasks {
                from_index: None,
                limit: None,
            },
        )
        .unwrap();

    assert_eq!(tasks_with_queries.len(), 1);
    assert_eq!(tasks.len(), 1);

    let mut has_created_hash: bool = false;
    for e in res.events {
        for a in e.attributes {
            if a.key == "with_queries" && a.value == "true" {
                has_created_hash = true;
            }
        }
    }

    res2.events.into_iter().any(|ev| {
        ev.attributes
            .into_iter()
            .any(|attr| attr.key == "with_queries" && attr.value == "false")
    });
    assert!(has_created_hash);
    Ok(())
}

#[test]
fn check_remove_create() -> StdResult<()> {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let validator = String::from("you");
    let amount = coin(3, NATIVE_DENOM);
    let stake = StakingMsg::Delegate { validator, amount };
    let msg: CosmosMsg = stake.clone().into();

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg,
                gas_limit: Some(150_000),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
            sender: None,
        },
    };

    // create a task
    let create_task_resp = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &create_task_msg,
            &coins(315006, NATIVE_DENOM),
        )
        .unwrap();

    let mut task_hash: String = String::new();
    for e in create_task_resp.events {
        for a in e.attributes {
            if a.key == "task_hash" && a.value.len() > 0 {
                task_hash = a.value;
            }
        }
    }

    println!("{:?}", task_hash);
    // check storage DOES have the task
    let new_task: Option<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTask {
                task_hash: task_hash.clone(),
            },
        )
        .unwrap();
    assert!(new_task.is_some());

    // Confirm slot exists, proving task was scheduled
    let slot_ids: GetSlotIdsResponse = app
        .wrap()
        .query_wasm_smart(&contract_addr.clone(), &QueryMsg::GetSlotIds {})
        .unwrap();
    let s_1: Vec<u64> = Vec::new();
    assert_eq!(s_1, slot_ids.time_ids);
    assert_eq!(vec![12346], slot_ids.block_ids);

    // Another person can't remove the task
    app.execute_contract(
        Addr::unchecked(ADMIN),
        contract_addr.clone(),
        &ExecuteMsg::RemoveTask {
            task_hash: task_hash.clone(),
        },
        &vec![],
    )
    .unwrap_err();

    // Remove the Task
    app.execute_contract(
        Addr::unchecked(ANYONE),
        contract_addr.clone(),
        &ExecuteMsg::RemoveTask {
            task_hash: task_hash.clone(),
        },
        &vec![],
    )
    .unwrap();

    // check storage DOESNT have the task
    let rem_task: Option<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTask {
                task_hash: task_hash.clone(),
            },
        )
        .unwrap();
    assert!(rem_task.is_none());

    // Check the contract total balance has decreased from the removed task
    let balances: GetBalancesResponse = app
        .wrap()
        .query_wasm_smart(&contract_addr.clone(), &QueryMsg::GetBalances {})
        .unwrap();
    assert_eq!(balances.available_balance.native, coins(1, NATIVE_DENOM));

    // Check the slots correctly removed the task
    let slot_ids: GetSlotIdsResponse = app
        .wrap()
        .query_wasm_smart(&contract_addr.clone(), &QueryMsg::GetSlotIds {})
        .unwrap();
    let s: Vec<u64> = Vec::new();
    assert_eq!(s.clone(), slot_ids.time_ids);
    assert_eq!(s, slot_ids.block_ids);

    Ok(())
}

#[test]
fn check_refill_create() -> StdResult<()> {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let validator = String::from("you");
    let amount = coin(3, NATIVE_DENOM);
    let stake = StakingMsg::Delegate { validator, amount };
    let msg: CosmosMsg = stake.clone().into();

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg,
                gas_limit: Some(150_000),
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
            sender: None,
        },
    };

    // create a task
    let create_task_resp = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &create_task_msg,
            &coins(315006, NATIVE_DENOM),
        )
        .unwrap();
    let mut task_hash: String = String::new();
    for e in create_task_resp.events {
        for a in e.attributes {
            if a.key == "task_hash" && a.value.len() > 0 {
                task_hash = a.value;
            }
        }
    }

    // refill task
    let res = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &ExecuteMsg::RefillTaskBalance {
                task_hash: task_hash.clone(),
            },
            &coins(3, NATIVE_DENOM),
        )
        .unwrap();
    // Assert returned event attributes include total
    let mut matches_new_totals: bool = false;
    for e in res.events {
        for a in e.attributes {
            if a.key == "total_deposit" && a.value == r#"["315009atom"]"#.to_string() {
                matches_new_totals = true;
            }
        }
    }
    assert!(matches_new_totals);

    // check the task totals
    let new_task: Option<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            &contract_addr.clone(),
            &QueryMsg::GetTask {
                task_hash: task_hash.clone(),
            },
        )
        .unwrap();
    assert!(new_task.is_some());

    if let Some(t) = new_task {
        assert_eq!(Addr::unchecked(ANYONE), t.owner_id);
        assert_eq!(coins(315009, NATIVE_DENOM), t.total_deposit);
    }

    // Check the balance has increased to include the new refilled total
    let balances: GetBalancesResponse = app
        .wrap()
        .query_wasm_smart(&contract_addr.clone(), &QueryMsg::GetBalances {})
        .unwrap();
    assert_eq!(
        coins(315010, NATIVE_DENOM),
        balances.available_balance.native
    );

    Ok(())
}

#[test]
fn check_gas_minimum() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let validator = String::from("you");
    let amount = coin(3, NATIVE_DENOM);
    let stake = StakingMsg::Delegate { validator, amount };
    let msg: CosmosMsg = stake.clone().into();
    let gas_limit = 150_000;
    let base_gas = GAS_BASE_FEE;

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
            sender: None,
        },
    };
    // create 1 token off task
    let gas_for_two = (base_gas + gas_limit) * 2;
    let enough_for_two = u128::from(
        (gas_for_two + gas_for_two * 5 / 100) * GAS_ADJUSTMENT_NUMERATOR_DEFAULT / GAS_DENOMINATOR
            * GAS_NUMERATOR_DEFAULT
            / GAS_DENOMINATOR
            + 3 * 2,
    );
    let res: ContractError = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &create_task_msg,
            &coins(enough_for_two - 1, NATIVE_DENOM),
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        res,
        ContractError::CoreError(CoreError::NotEnoughNative {
            denom: NATIVE_DENOM.to_string(),
            lack: Uint128::from(1u128)
        })
    );

    // create a task
    let res = app.execute_contract(
        Addr::unchecked(ANYONE),
        contract_addr.clone(),
        &create_task_msg,
        &coins(enough_for_two, NATIVE_DENOM),
    );
    assert!(res.is_ok());
}

#[test]
fn check_gas_default() {
    let (mut app, cw_template_contract, _) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let validator = String::from("you");
    let amount = coin(3, NATIVE_DENOM);
    let stake = StakingMsg::Delegate { validator, amount };
    let msg: CosmosMsg = stake.clone().into();
    let gas_limit = GAS_ACTION_FEE;
    let base_gas = GAS_BASE_FEE;
    // let send = BankMsg::Send {
    //     to_address: validator,
    //     amount: vec![amount],
    // };

    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg,
                gas_limit: None,
            }],
            queries: None,
            transforms: None,
            cw20_coins: vec![],
            sender: None,
        },
    };
    // create 1 token off task
    // for one task need gas + staking amount

    let gas_for_one = base_gas + gas_limit;
    let gas_for_one_with_fee = gas_for_one + gas_for_one * 5 / 100;
    let enough_for_two = 2 * u128::from(
        gas_for_one_with_fee * GAS_ADJUSTMENT_NUMERATOR_DEFAULT / GAS_DENOMINATOR
            * GAS_NUMERATOR_DEFAULT
            / GAS_DENOMINATOR
            + 3,
    );

    let res: ContractError = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &create_task_msg,
            &coins(enough_for_two - 1, NATIVE_DENOM),
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        res,
        ContractError::CoreError(CoreError::NotEnoughNative {
            denom: NATIVE_DENOM.to_string(),
            lack: Uint128::from(1u128)
        })
    );

    // create a task
    // for Immediate task must attach amount for two times execution
    let res = app.execute_contract(
        Addr::unchecked(ANYONE),
        contract_addr.clone(),
        &create_task_msg,
        &coins(enough_for_two, NATIVE_DENOM),
    );
    assert!(res.is_ok());
}
