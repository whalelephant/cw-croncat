use cosmwasm_std::{coins, from_binary, to_binary, Addr, BankMsg, Uint128};
use croncat_sdk_manager::types::{gas_price_defaults, Config, UpdateConfig};
use croncat_sdk_tasks::types::{Action, Interval, TaskResponse};
use cw20::Cw20CoinVerified;
use cw_storage_plus::KeyDeserialize;

use crate::{
    contract::DEFAULT_FEE,
    msg::{ExecuteMsg, InstantiateMsg, ReceiveMsg},
    tests::{
        helpers::{default_app, default_instantiate_message, init_manager, query_manager_config},
        helpers::{init_factory, query_manager_balances},
        ADMIN, AGENT1, AGENT2, ANYONE, DENOM,
    },
    ContractError,
};
use cosmwasm_std::{coin, StdError};
use croncat_sdk_manager::types::GasPrice;
use cw_multi_test::{BankSudo, Executor};

use super::{
    contracts,
    helpers::{init_agents, init_tasks},
    PARTICIPANT0,
};
use super::{
    helpers::{activate_agent, add_little_time, init_cw20, query_users_manager},
    AGENT0,
};

mod instantiate_tests {
    use super::*;

    #[test]
    fn default_init() {
        let mut app = default_app();
        let instantiate_msg: InstantiateMsg = default_instantiate_message();
        let factory_addr = init_factory(&mut app);

        let manager_addr = init_manager(&mut app, &instantiate_msg, &factory_addr);
        let config = query_manager_config(&app, &manager_addr);

        let expected_config = Config {
            paused: false,
            owner_addr: Addr::unchecked(ADMIN),
            croncat_factory_addr: factory_addr,
            croncat_tasks_key: ("tasks".to_owned(), [0, 1]),
            croncat_agents_key: ("agents".to_owned(), [0, 1]),
            agent_fee: DEFAULT_FEE,
            treasury_fee: DEFAULT_FEE,
            gas_price: Default::default(),
            cw20_whitelist: vec![],
            native_denom: DENOM.to_owned(),
            limit: 100,
            treasury_addr: None,
        };
        assert_eq!(config, expected_config);
    }

    #[test]
    fn custom_init() {
        let mut app = default_app();
        let factory_addr = init_factory(&mut app);

        let instantiate_msg: InstantiateMsg = InstantiateMsg {
            denom: "cron".to_owned(),
            version: Some("0.1".to_owned()),
            croncat_tasks_key: (AGENT1.to_owned(), [0, 1]),
            croncat_agents_key: (AGENT2.to_owned(), [0, 1]),
            owner_addr: Some(ANYONE.to_owned()),
            gas_price: Some(GasPrice {
                numerator: 10,
                denominator: 20,
                gas_adjustment_numerator: 30,
            }),
            treasury_addr: Some(AGENT2.to_owned()),
        };
        let attach_funds = vec![coin(5000, "denom"), coin(2400, DENOM)];
        app.sudo(
            BankSudo::Mint {
                to_address: ADMIN.to_owned(),
                amount: attach_funds.clone(),
            }
            .into(),
        )
        .unwrap();
        let manager_addr = init_manager(&mut app, &instantiate_msg, &factory_addr);

        let config = query_manager_config(&app, &manager_addr);

        let expected_config = Config {
            paused: false,
            owner_addr: Addr::unchecked(ANYONE),
            croncat_factory_addr: factory_addr,
            croncat_tasks_key: (AGENT1.to_owned(), [0, 1]),
            croncat_agents_key: (AGENT2.to_owned(), [0, 1]),
            agent_fee: DEFAULT_FEE,
            treasury_fee: DEFAULT_FEE,
            gas_price: GasPrice {
                numerator: 10,
                denominator: 20,
                gas_adjustment_numerator: 30,
            },
            cw20_whitelist: vec![],
            native_denom: "cron".to_owned(),
            limit: 100,
            treasury_addr: Some(Addr::unchecked(AGENT2)),
        };
        assert_eq!(config, expected_config);

        let manager_balances = query_manager_balances(&app, &manager_addr);
        assert_eq!(manager_balances, Uint128::zero());
    }

    #[test]
    fn invalid_inits() {
        let mut app = default_app();
        let code_id = app.store_code(contracts::croncat_manager_contract());
        // Invalid gas price
        let instantiate_msg: InstantiateMsg = InstantiateMsg {
            gas_price: Some(GasPrice {
                numerator: 0,
                denominator: 1,
                gas_adjustment_numerator: 2,
            }),
            ..default_instantiate_message()
        };

        let error: ContractError = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(ADMIN),
                &instantiate_msg,
                &[],
                "croncat-manager",
                None,
            )
            .unwrap_err()
            .downcast()
            .unwrap();
        assert_eq!(error, ContractError::InvalidGasPrice {});

        // Bad owner_addr
        let instantiate_msg: InstantiateMsg = InstantiateMsg {
            owner_addr: Some("BAD_INPUT".to_owned()),
            ..default_instantiate_message()
        };

        let error: ContractError = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(ADMIN),
                &instantiate_msg,
                &[],
                "croncat-manager",
                None,
            )
            .unwrap_err()
            .downcast()
            .unwrap();
        assert_eq!(
            error,
            ContractError::Std(StdError::generic_err(
                "Invalid input: address not normalized"
            ))
        );
    }
}

#[test]
fn update_config() {
    let mut app = default_app();
    let factory_addr = init_factory(&mut app);

    let instantiate_msg: InstantiateMsg = default_instantiate_message();

    let attach_funds = vec![coin(5000, "denom"), coin(2400, DENOM)];
    app.sudo(
        BankSudo::Mint {
            to_address: ADMIN.to_owned(),
            amount: attach_funds.clone(),
        }
        .into(),
    )
    .unwrap();

    let manager_addr = init_manager(&mut app, &instantiate_msg, &factory_addr);

    let update_cfg_msg = UpdateConfig {
        owner_addr: Some("new_owner".to_string()),
        paused: Some(true),
        agent_fee: Some(0),
        treasury_fee: Some(0),
        gas_price: Some(GasPrice {
            numerator: 555,
            denominator: 666,
            gas_adjustment_numerator: 777,
        }),
        croncat_tasks_key: Some(("new_key_tasks".to_owned(), [0, 1])),
        croncat_agents_key: Some(("new_key_agents".to_owned(), [0, 1])),
        treasury_addr: Some(ANYONE.to_owned()),
    };

    app.execute_contract(
        Addr::unchecked(ADMIN),
        manager_addr.clone(),
        &ExecuteMsg::UpdateConfig(Box::new(update_cfg_msg)),
        &[],
    )
    .unwrap();
    let config = query_manager_config(&app, &manager_addr);
    let expected_config = Config {
        paused: true,
        owner_addr: Addr::unchecked("new_owner"),
        croncat_factory_addr: factory_addr,
        croncat_tasks_key: ("new_key_tasks".to_owned(), [0, 1]),
        croncat_agents_key: ("new_key_agents".to_owned(), [0, 1]),
        agent_fee: 0,
        treasury_fee: 0,
        gas_price: GasPrice {
            numerator: 555,
            denominator: 666,
            gas_adjustment_numerator: 777,
        },
        cw20_whitelist: vec![],
        native_denom: DENOM.to_owned(),
        limit: 100,
        treasury_addr: Some(Addr::unchecked(ANYONE)),
    };
    assert_eq!(config, expected_config);

    // Shouldn't override any fields to None or anything
    let update_cfg_msg = UpdateConfig {
        owner_addr: None,
        paused: None,
        agent_fee: None,
        treasury_fee: None,
        gas_price: None,
        croncat_tasks_key: None,
        croncat_agents_key: None,
        treasury_addr: None,
    };

    app.execute_contract(
        Addr::unchecked("new_owner"),
        manager_addr.clone(),
        &ExecuteMsg::UpdateConfig(Box::new(update_cfg_msg)),
        &[],
    )
    .unwrap();
    let config = query_manager_config(&app, &manager_addr);
    assert_eq!(config, expected_config);
}

#[test]
fn invalid_updates_config() {
    let mut app = default_app();
    let instantiate_msg: InstantiateMsg = default_instantiate_message();
    let factory_addr = init_factory(&mut app);

    let attach_funds = vec![coin(5000, "denom"), coin(2400, DENOM)];
    app.sudo(
        BankSudo::Mint {
            to_address: ADMIN.to_owned(),
            amount: attach_funds.clone(),
        }
        .into(),
    )
    .unwrap();

    let manager_addr = init_manager(&mut app, &instantiate_msg, &factory_addr);

    // Unauthorized
    let update_cfg_msg = UpdateConfig {
        owner_addr: Some("new_owner".to_string()),
        paused: Some(true),
        agent_fee: Some(0),
        treasury_fee: Some(2),
        gas_price: Some(GasPrice {
            numerator: 555,
            denominator: 666,
            gas_adjustment_numerator: 777,
        }),
        croncat_tasks_key: Some(("new_key_tasks".to_owned(), [0, 1])),
        croncat_agents_key: Some(("new_key_agents".to_owned(), [0, 1])),
        treasury_addr: Some(ANYONE.to_owned()),
    };
    let err: ContractError = app
        .execute_contract(
            // Not admin
            Addr::unchecked(ANYONE),
            manager_addr.clone(),
            &ExecuteMsg::UpdateConfig(Box::new(update_cfg_msg)),
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ContractError::Unauthorized {});

    // Invalid gas_price
    let update_cfg_msg = UpdateConfig {
        owner_addr: Some("new_owner".to_string()),
        paused: Some(true),
        agent_fee: Some(0),
        treasury_fee: Some(2),
        gas_price: Some(GasPrice {
            numerator: 555,
            denominator: 0,
            gas_adjustment_numerator: 777,
        }),
        croncat_tasks_key: Some(("new_key_tasks".to_owned(), [0, 1])),
        croncat_agents_key: Some(("new_key_agents".to_owned(), [0, 1])),
        treasury_addr: Some(ANYONE.to_owned()),
    };
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            manager_addr.clone(),
            &ExecuteMsg::UpdateConfig(Box::new(update_cfg_msg)),
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ContractError::InvalidGasPrice {});

    // Invalid owner
    let update_cfg_msg = UpdateConfig {
        owner_addr: Some("New_owner".to_string()),
        paused: Some(true),
        agent_fee: Some(0),
        treasury_fee: Some(2),
        gas_price: Some(GasPrice {
            numerator: 555,
            denominator: 666,
            gas_adjustment_numerator: 777,
        }),
        croncat_tasks_key: Some(("new_key_tasks".to_owned(), [0, 1])),
        croncat_agents_key: Some(("new_key_agents".to_owned(), [0, 1])),
        treasury_addr: Some(ANYONE.to_owned()),
    };
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            manager_addr,
            &ExecuteMsg::UpdateConfig(Box::new(update_cfg_msg)),
            &[],
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
fn cw20_receive() {
    let mut app = default_app();
    let factory_addr = init_factory(&mut app);

    let instantiate_msg: InstantiateMsg = default_instantiate_message();
    let manager_addr = init_manager(&mut app, &instantiate_msg, &factory_addr);

    let cw20_addr = init_cw20(&mut app);
    app.execute_contract(
        Addr::unchecked(ADMIN),
        cw20_addr.clone(),
        &cw20::Cw20ExecuteMsg::Send {
            contract: manager_addr.to_string(),
            amount: Uint128::new(555),
            msg: to_binary(&ReceiveMsg::RefillTempBalance {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    let wallet_balances = query_users_manager(&app, &manager_addr, ADMIN);
    assert_eq!(
        wallet_balances,
        vec![Cw20CoinVerified {
            address: cw20_addr,
            amount: Uint128::new(555),
        }]
    );
}

#[test]
fn cw20_bad_messages() {
    let mut app = default_app();
    let factory_addr = init_factory(&mut app);

    let instantiate_msg: InstantiateMsg = default_instantiate_message();
    let manager_addr = init_manager(&mut app, &instantiate_msg, &factory_addr);

    let cw20_addr = init_cw20(&mut app);
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            cw20_addr.clone(),
            &cw20::Cw20ExecuteMsg::Send {
                contract: manager_addr.to_string(),
                amount: Uint128::new(555),
                msg: Default::default(),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        err,
        ContractError::Std(StdError::parse_err(
            "croncat_sdk_manager::msg::ManagerReceiveMsg",
            "EOF while parsing a JSON value."
        ))
    );

    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            cw20_addr,
            &cw20::Cw20ExecuteMsg::Send {
                contract: manager_addr.to_string(),
                amount: Uint128::new(555),
                msg: to_binary(&true).unwrap(),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        err,
        ContractError::Std(StdError::parse_err(
            "croncat_sdk_manager::msg::ManagerReceiveMsg",
            "Expected to parse either a `true`, `false`, or a `null`."
        ))
    );
}

#[test]
fn users_withdraws() {
    let mut app = default_app();
    let factory_addr = init_factory(&mut app);

    let instantiate_msg: InstantiateMsg = default_instantiate_message();
    let manager_addr = init_manager(&mut app, &instantiate_msg, &factory_addr);

    // refill balances
    let cw20_addr = init_cw20(&mut app);
    app.execute_contract(
        Addr::unchecked(ADMIN),
        cw20_addr.clone(),
        &cw20::Cw20ExecuteMsg::Send {
            contract: manager_addr.to_string(),
            amount: Uint128::new(1000),
            msg: to_binary(&ReceiveMsg::RefillTempBalance {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    // Withdraw half
    let user_cw20_balance: cw20::BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            cw20_addr.clone(),
            &cw20::Cw20QueryMsg::Balance {
                address: ADMIN.to_owned(),
            },
        )
        .unwrap();

    app.execute_contract(
        Addr::unchecked(ADMIN),
        manager_addr.clone(),
        &ExecuteMsg::UserWithdraw { limit: None },
        &[],
    )
    .unwrap();

    // Check it got withdrawn

    // Check it updated on manager
    let manager_wallet_balance = query_users_manager(&app, &manager_addr, ADMIN);
    assert_eq!(manager_wallet_balance, vec![]);

    let fully_withdrawn_user_balance: cw20::BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            cw20_addr,
            &cw20::Cw20QueryMsg::Balance {
                address: ADMIN.to_owned(),
            },
        )
        .unwrap();
    assert_eq!(
        fully_withdrawn_user_balance.balance,
        user_cw20_balance.balance + Uint128::new(1000)
    );
}

#[test]
fn failed_users_withdraws() {
    let mut app = default_app();
    let factory_addr = init_factory(&mut app);

    let instantiate_msg: InstantiateMsg = default_instantiate_message();
    let manager_addr = init_manager(&mut app, &instantiate_msg, &factory_addr);

    let cw20_addr = init_cw20(&mut app);

    // try to withdraw empty balances
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            manager_addr.clone(),
            &ExecuteMsg::UserWithdraw { limit: None },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, ContractError::EmptyBalance {});

    // refill balances
    app.execute_contract(
        Addr::unchecked(ADMIN),
        cw20_addr,
        &cw20::Cw20ExecuteMsg::Send {
            contract: manager_addr.to_string(),
            amount: Uint128::new(1000),
            msg: to_binary(&ReceiveMsg::RefillTempBalance {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    // Another user tries to withdraw
    // No steals here
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            manager_addr,
            &ExecuteMsg::UserWithdraw { limit: None },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, ContractError::EmptyBalance {});
}

#[test]
fn withdraw_balances() {
    let mut app = default_app();
    let factory_addr = init_factory(&mut app);

    let instantiate_msg: InstantiateMsg = default_instantiate_message();

    let manager_addr = init_manager(&mut app, &instantiate_msg, &factory_addr);

    // refill balance
    let cw20_addr = init_cw20(&mut app);
    app.execute_contract(
        Addr::unchecked(ADMIN),
        cw20_addr,
        &cw20::Cw20ExecuteMsg::Send {
            contract: manager_addr.to_string(),
            amount: Uint128::new(1000),
            msg: to_binary(&ReceiveMsg::RefillTempBalance {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    // Withdraw all of balances
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            manager_addr,
            &ExecuteMsg::OwnerWithdraw {},
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(err, ContractError::EmptyBalance {});

    // TODO: after split tasks
}

#[test]
fn failed_move_balances() {
    let mut app = default_app();
    let factory_addr = init_factory(&mut app);

    let instantiate_msg: InstantiateMsg = default_instantiate_message();
    let manager_addr = init_manager(&mut app, &instantiate_msg, &factory_addr);

    let attach_funds = vec![coin(2400, DENOM), coin(5000, "denom")];
    app.sudo(
        BankSudo::Mint {
            to_address: ADMIN.to_owned(),
            amount: attach_funds.clone(),
        }
        .into(),
    )
    .unwrap();

    // refill balance
    let cw20_addr = init_cw20(&mut app);
    app.execute_contract(
        Addr::unchecked(ADMIN),
        cw20_addr,
        &cw20::Cw20ExecuteMsg::Send {
            contract: manager_addr.to_string(),
            amount: Uint128::new(1000),
            msg: to_binary(&ReceiveMsg::RefillTempBalance {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    // Withdraw not by owner
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            manager_addr,
            &ExecuteMsg::OwnerWithdraw {},
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn simple_bank_transfer_execution() {
    let mut app = default_app();
    let factory_addr = init_factory(&mut app);

    let instantiate_msg: InstantiateMsg = default_instantiate_message();
    let manager_addr = init_manager(&mut app, &instantiate_msg, &factory_addr);
    let agents_addr = init_agents(&mut app, &factory_addr);
    let tasks_addr = init_tasks(&mut app, &factory_addr);

    activate_agent(&mut app, &agents_addr);

    let task = croncat_sdk_tasks::types::TaskRequest {
        interval: Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: BankMsg::Send {
                to_address: "bob".to_owned(),
                amount: coins(45, DENOM),
            }
            .into(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20: None,
    };
    let res = app
        .execute_contract(
            Addr::unchecked(PARTICIPANT0),
            tasks_addr.clone(),
            &croncat_sdk_tasks::msg::TasksExecuteMsg::CreateTask {
                task: Box::new(task),
            },
            &coins(600_000, DENOM),
        )
        .unwrap();
    let task_hash = String::from_vec(res.data.unwrap().0).unwrap();
    let task: Option<TaskResponse> = app
        .wrap()
        .query_wasm_smart(
            tasks_addr.clone(),
            &croncat_tasks::msg::QueryMsg::Task {
                task_hash: task_hash.clone(),
            },
        )
        .unwrap();

    let expected_gone_amount = {
        let gas_needed = task.unwrap().amount_for_one_task.gas as f64 * 1.5;
        let gas_fees = gas_needed * ((DEFAULT_FEE as f64 + DEFAULT_FEE as f64) / 100.0);
        let amount_for_task = gas_needed * 0.04;
        let amount_for_fees = gas_fees * 0.04;
        amount_for_task + amount_for_fees + 45.0
    } as u128;

    app.update_block(add_little_time);

    let participant_balance = app.wrap().query_balance(PARTICIPANT0, DENOM).unwrap();
    app.execute_contract(
        Addr::unchecked(AGENT0),
        manager_addr.clone(),
        &ExecuteMsg::ProxyCall { task_hash: None },
        &[],
    )
    .unwrap();

    let bob_balances = app.wrap().query_all_balances("bob").unwrap();
    assert_eq!(bob_balances, coins(45, DENOM));

    let after_unregister_participant_balance =
        app.wrap().query_balance(PARTICIPANT0, DENOM).unwrap();
    assert_eq!(
        600_000 - expected_gone_amount,
        after_unregister_participant_balance.amount.u128() - participant_balance.amount.u128()
    );
}

//TODO: this test is failing as no factory contract is initialized
#[test]
fn test_should_fail_with_zero_rewards() {
    let mut app = default_app();
    let factory_addr = init_factory(&mut app);

    let instantiate_msg: InstantiateMsg = default_instantiate_message();
    let _agents_addr = init_agents(&mut app, &factory_addr);
    let manager_addr = init_manager(&mut app, &instantiate_msg, &factory_addr);

    //No available rewards for withdraw
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ANYONE),
            manager_addr,
            &ExecuteMsg::WithdrawAgentRewards(None),
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ContractError::NoRewardsOwnerAgentFound {});
}
