use crate::tests::helpers::{add_little_time, proper_instantiate};
use crate::ContractError;
use cosmwasm_std::{coins, to_binary, Addr, CosmosMsg, StdError, Uint128, WasmMsg};
use cw20::{BalanceResponse, Cw20Coin, Cw20CoinVerified};
use cw_croncat_core::error::CoreError;
use cw_croncat_core::msg::{
    ExecuteMsg, GetWalletBalancesResponse, QueryMsg, TaskRequest, TaskResponse,
};
use cw_croncat_core::types::{Action, Interval};
use cw_multi_test::Executor;

use super::helpers::{AGENT0, AGENT_BENEFICIARY, ANYONE, NATIVE_DENOM};

#[test]
fn test_cw20_action() {
    let (mut app, cw_template_contract, cw20_contract) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    // fill balance of cw20 tokens of user
    let user = ANYONE;
    let refill_balance_msg = cw20::Cw20ExecuteMsg::Send {
        contract: contract_addr.to_string(),
        amount: 10u128.into(),
        msg: Default::default(),
    };
    app.execute_contract(
        Addr::unchecked(user),
        cw20_contract.clone(),
        &refill_balance_msg,
        &[],
    )
    .unwrap();

    // create a task sending cw20 to AGENT0
    let msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: cw20_contract.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
            recipient: AGENT0.to_string(),
            amount: 10u128.into(),
        })
        .unwrap(),
        funds: vec![],
    }
    .into();
    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Once,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: msg.clone(),
                gas_limit: Some(150_000),
            }],
            rules: None,
            cw20_coins: vec![Cw20Coin {
                address: cw20_contract.to_string(),
                amount: 10u128.into(),
            }],
        },
    };
    app.execute_contract(
        Addr::unchecked(user),
        contract_addr.clone(),
        &create_task_msg,
        &coins(u128::from(300_010_u128), NATIVE_DENOM),
    )
    .unwrap();

    // quick agent register
    {
        let msg = ExecuteMsg::RegisterAgent {
            payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
        };
        app.execute_contract(Addr::unchecked(AGENT0), contract_addr.clone(), &msg, &[])
            .unwrap();
    }

    app.update_block(add_little_time);

    // Agent executes transfer
    let proxy_call_msg = ExecuteMsg::ProxyCall { task_hash: None };
    app.execute_contract(
        Addr::unchecked(AGENT0),
        contract_addr.clone(),
        &proxy_call_msg,
        &vec![],
    )
    .unwrap();

    // Check new balance of AGENT0
    let balance: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            cw20_contract,
            &cw20::Cw20QueryMsg::Balance {
                address: AGENT0.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        balance,
        BalanceResponse {
            balance: 10_u128.into()
        }
    );
}

#[test]
fn test_cw20_balances() {
    let (mut app, cw_template_contract, cw20_contract) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    // fill balance of cw20 tokens of user
    let user = ANYONE;
    // Balances before refill
    let balances: GetWalletBalancesResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetWalletBalances {
                wallet: user.to_string(),
            },
        )
        .unwrap();
    assert!(balances.cw20_balances.is_empty());

    let refill_balance_msg = cw20::Cw20ExecuteMsg::Send {
        contract: contract_addr.to_string(),
        amount: 10u128.into(),
        msg: Default::default(),
    };
    app.execute_contract(
        Addr::unchecked(user),
        cw20_contract.clone(),
        &refill_balance_msg,
        &[],
    )
    .unwrap();

    // Check Balances of user after refill
    let balances: GetWalletBalancesResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetWalletBalances {
                wallet: user.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        balances,
        GetWalletBalancesResponse {
            cw20_balances: vec![Cw20CoinVerified {
                address: cw20_contract.clone(),
                amount: 10u128.into()
            }]
        }
    );

    // create a task sending cw20 to AGENT0
    let msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: cw20_contract.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
            recipient: AGENT0.to_string(),
            amount: 10u128.into(),
        })
        .unwrap(),
        funds: vec![],
    }
    .into();
    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Once,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: msg.clone(),
                gas_limit: Some(150_000),
            }],
            rules: None,
            cw20_coins: vec![Cw20Coin {
                address: cw20_contract.to_string(),
                amount: 10u128.into(),
            }],
        },
    };
    let mut resp = app
        .execute_contract(
            Addr::unchecked(user),
            contract_addr.clone(),
            &create_task_msg,
            &coins(u128::from(300_010_u128), NATIVE_DENOM),
        )
        .unwrap();
    let task_hash = resp
        .events
        .pop()
        .unwrap()
        .attributes
        .into_iter()
        .find(|attr| attr.key == "task_hash")
        .unwrap();

    // Check task balances increased
    let task: TaskResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetTask {
                task_hash: task_hash.value,
            },
        )
        .unwrap();
    assert_eq!(
        task.total_cw20_deposit,
        vec![Cw20CoinVerified {
            address: cw20_contract.clone(),
            amount: 10u128.into()
        }]
    );
    // And user balances decreased
    let balances: GetWalletBalancesResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::GetWalletBalances {
                wallet: user.to_string(),
            },
        )
        .unwrap();
    assert!(balances.cw20_balances.is_empty());
}

#[test]
fn test_cw20_negative() {
    let (mut app, cw_template_contract, cw20_contract) = proper_instantiate();
    let contract_addr = cw_template_contract.addr();

    let user = ANYONE;

    // create a task with empty balance
    let msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: cw20_contract.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
            recipient: AGENT0.to_string(),
            amount: 10u128.into(),
        })
        .unwrap(),
        funds: vec![],
    }
    .into();
    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
            interval: Interval::Immediate,
            boundary: None,
            stop_on_fail: false,
            actions: vec![Action {
                msg: msg.clone(),
                gas_limit: Some(150_000),
            }],
            rules: None,
            cw20_coins: vec![Cw20Coin {
                address: cw20_contract.to_string(),
                amount: 10u128.into(),
            }],
        },
    };
    let resp: ContractError = app
        .execute_contract(
            Addr::unchecked(user),
            contract_addr.clone(),
            &create_task_msg,
            &coins(u128::from(300_010_u128), NATIVE_DENOM),
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(resp, ContractError::CoreError(CoreError::EmptyBalance {}));
    // or with not enough balance

    // fill balance of cw20 tokens of user
    let refill_balance_msg = cw20::Cw20ExecuteMsg::Send {
        contract: contract_addr.to_string(),
        amount: 9u128.into(),
        msg: Default::default(),
    };
    app.execute_contract(
        Addr::unchecked(user),
        cw20_contract.clone(),
        &refill_balance_msg,
        &[],
    )
    .unwrap();

    let resp: ContractError = app
        .execute_contract(
            Addr::unchecked(user),
            contract_addr.clone(),
            &create_task_msg,
            &coins(u128::from(315_000_u128), "atom"),
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert!(matches!(
        resp,
        ContractError::CoreError(CoreError::Std(StdError::Overflow { .. }))
    ));

    // Create a task that does cw20 transfer without attaching cw20 to the task
    let create_task_msg = ExecuteMsg::CreateTask {
        task: TaskRequest {
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
    };
    let resp: ContractError = app
        .execute_contract(
            Addr::unchecked(user),
            contract_addr.clone(),
            &create_task_msg,
            &coins(u128::from(315_000_u128), "atom"),
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    println!("resp: {resp:?}");
    assert!(matches!(
                resp,
                ContractError::CoreError(CoreError::NotEnoughCw20 { lack, .. }) if lack == Uint128::from(10_u128)));
}
