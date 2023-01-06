use cosmwasm_std::{coins, to_binary, Addr, Uint128};
use croncat_sdk_core::{
    balancer::{BalancerMode, RoundRobinBalancer},
    types::{BalancesResponse, Config, UpdateConfig},
};
use cw20::{Cw20Coin, Cw20CoinVerified};

use crate::{
    contract::{
        DEFAULT_NOMINATION_DURATION, GAS_ACTION_FEE, GAS_BASE_FEE, GAS_QUERY_FEE,
        GAS_WASM_QUERY_FEE,
    },
    msg::{ExecuteMsg, InstantiateMsg, ReceiveMsg},
    tests::{
        helpers::query_manager_balances,
        helpers::{default_app, default_instantiate_message, init_manager, query_manager_config},
        ADMIN, AGENT0, AGENT1, AGENT2, ANYONE, DENOM,
    },
    ContractError,
};
use cosmwasm_std::{coin, StdError, Uint64};
use croncat_sdk_core::types::GasPrice;
use cw_multi_test::{BankSudo, Executor};

use super::helpers::{init_cw20, query_cw20_wallet_manager};

mod instantiate_tests {
    use super::*;

    #[test]
    fn default_init() {
        let mut app = default_app();
        let instantiate_msg: InstantiateMsg = default_instantiate_message();

        let manager_addr = init_manager(&mut app, instantiate_msg, &[]).unwrap();
        let config = query_manager_config(&app, &manager_addr);

        let expected_config = Config {
            paused: false,
            owner_id: Addr::unchecked(ADMIN),
            min_tasks_per_agent: 3,
            agents_eject_threshold: 600,
            agent_nomination_duration: DEFAULT_NOMINATION_DURATION,
            cw_rules_addr: Addr::unchecked("cw_rules_addr"),
            croncat_tasks_addr: Addr::unchecked("croncat_tasks_addr"),
            croncat_agents_addr: Addr::unchecked("croncat_agents_addr"),
            agent_fee: 5,
            gas_price: Default::default(),
            gas_base_fee: GAS_BASE_FEE,
            gas_action_fee: GAS_ACTION_FEE,
            gas_query_fee: GAS_QUERY_FEE,
            gas_wasm_query_fee: GAS_WASM_QUERY_FEE,
            slot_granularity_time: 10_000_000_000,
            cw20_whitelist: vec![],
            native_denom: DENOM.to_owned(),
            balancer: Default::default(),
            limit: 100,
        };
        assert_eq!(config, expected_config)
    }

    #[test]
    fn custom_init() {
        let mut app = default_app();
        let instantiate_msg: InstantiateMsg = InstantiateMsg {
            denom: "cron".to_owned(),
            cw_rules_addr: AGENT0.to_owned(),
            croncat_tasks_addr: AGENT1.to_owned(),
            croncat_agents_addr: AGENT2.to_owned(),
            owner_id: Some(ANYONE.to_owned()),
            gas_base_fee: Some(Uint64::new(1001)),
            gas_action_fee: Some(Uint64::new(2002)),
            gas_query_fee: Some(Uint64::new(3003)),
            gas_wasm_query_fee: Some(Uint64::new(4004)),
            gas_price: Some(GasPrice {
                numerator: 10,
                denominator: 20,
                gas_adjustment_numerator: 30,
            }),
            agent_nomination_duration: Some(20),
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
        let manager_addr = init_manager(&mut app, instantiate_msg, &attach_funds).unwrap();

        let config = query_manager_config(&app, &manager_addr);

        let expected_config = Config {
            paused: false,
            owner_id: Addr::unchecked(ANYONE),
            min_tasks_per_agent: 3,
            agents_eject_threshold: 600,
            agent_nomination_duration: 20,
            cw_rules_addr: Addr::unchecked(AGENT0),
            croncat_tasks_addr: Addr::unchecked(AGENT1),
            croncat_agents_addr: Addr::unchecked(AGENT2),
            agent_fee: 5,
            gas_price: GasPrice {
                numerator: 10,
                denominator: 20,
                gas_adjustment_numerator: 30,
            },
            gas_base_fee: 1001,
            gas_action_fee: 2002,
            gas_query_fee: 3003,
            gas_wasm_query_fee: 4004,
            slot_granularity_time: 10_000_000_000,
            cw20_whitelist: vec![],
            native_denom: "cron".to_owned(),
            balancer: Default::default(),
            limit: 100,
        };
        assert_eq!(config, expected_config);

        let manager_balances = query_manager_balances(&app, &manager_addr);
        for coin in attach_funds {
            assert!(manager_balances.available_native_balance.contains(&coin))
        }
        assert_eq!(manager_balances.available_cw20_balance, vec![]);
    }

    #[test]
    fn invalid_inits() {
        let mut app = default_app();

        // Invalid gas price
        let instantiate_msg: InstantiateMsg = InstantiateMsg {
            gas_price: Some(GasPrice {
                numerator: 0,
                denominator: 1,
                gas_adjustment_numerator: 2,
            }),
            ..default_instantiate_message()
        };

        let error: ContractError = init_manager(&mut app, instantiate_msg, &[])
            .unwrap_err()
            .downcast()
            .unwrap();
        assert_eq!(error, ContractError::InvalidGasPrice {});

        // Bad owner_id
        let instantiate_msg: InstantiateMsg = InstantiateMsg {
            owner_id: Some("BAD_INPUT".to_owned()),
            ..default_instantiate_message()
        };

        let error: ContractError = init_manager(&mut app, instantiate_msg, &[])
            .unwrap_err()
            .downcast()
            .unwrap();
        assert_eq!(
            error,
            ContractError::Std(StdError::generic_err(
                "Invalid input: address not normalized"
            ))
        );

        // Bad cw_rules_addr
        let instantiate_msg: InstantiateMsg = InstantiateMsg {
            cw_rules_addr: "BAD_INPUT".to_owned(),
            ..default_instantiate_message()
        };

        let error: ContractError = init_manager(&mut app, instantiate_msg, &[])
            .unwrap_err()
            .downcast()
            .unwrap();
        assert_eq!(
            error,
            ContractError::Std(StdError::generic_err(
                "Invalid input: address not normalized"
            ))
        );

        // Bad croncat_tasks_addr
        let instantiate_msg: InstantiateMsg = InstantiateMsg {
            croncat_tasks_addr: "BAD_INPUT".to_owned(),
            ..default_instantiate_message()
        };

        let error: ContractError = init_manager(&mut app, instantiate_msg, &[])
            .unwrap_err()
            .downcast()
            .unwrap();
        assert_eq!(
            error,
            ContractError::Std(StdError::generic_err(
                "Invalid input: address not normalized"
            ))
        );

        // Bad croncat_agents_addr
        let instantiate_msg: InstantiateMsg = InstantiateMsg {
            croncat_agents_addr: "BAD_INPUT".to_owned(),
            ..default_instantiate_message()
        };

        let error: ContractError = init_manager(&mut app, instantiate_msg, &[])
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

    let manager_addr = init_manager(&mut app, instantiate_msg, &attach_funds).unwrap();

    let update_cfg_msg = UpdateConfig {
        owner_id: Some("new_owner".to_string()),
        slot_granularity_time: Some(1234),
        paused: Some(true),
        agent_fee: Some(0),
        gas_base_fee: Some(1111),
        gas_action_fee: Some(2222),
        gas_query_fee: Some(3333),
        gas_wasm_query_fee: Some(4444),
        gas_price: Some(GasPrice {
            numerator: 555,
            denominator: 666,
            gas_adjustment_numerator: 777,
        }),
        min_tasks_per_agent: Some(1),
        agents_eject_threshold: Some(3),
        balancer: Some(RoundRobinBalancer::new(BalancerMode::Equalizer)),
    };

    app.execute_contract(
        Addr::unchecked(ADMIN),
        manager_addr.clone(),
        &ExecuteMsg::UpdateConfig(update_cfg_msg),
        &[],
    )
    .unwrap();
    let config = query_manager_config(&app, &manager_addr);
    let expected_config = Config {
        paused: true,
        owner_id: Addr::unchecked("new_owner"),
        min_tasks_per_agent: 1,
        agents_eject_threshold: 3,
        agent_nomination_duration: DEFAULT_NOMINATION_DURATION,
        cw_rules_addr: Addr::unchecked("cw_rules_addr"),
        croncat_tasks_addr: Addr::unchecked("croncat_tasks_addr"),
        croncat_agents_addr: Addr::unchecked("croncat_agents_addr"),
        agent_fee: 0,
        gas_price: GasPrice {
            numerator: 555,
            denominator: 666,
            gas_adjustment_numerator: 777,
        },
        gas_base_fee: 1111,
        gas_action_fee: 2222,
        gas_query_fee: 3333,
        gas_wasm_query_fee: 4444,
        slot_granularity_time: 1234,
        cw20_whitelist: vec![],
        native_denom: DENOM.to_owned(),
        balancer: RoundRobinBalancer {
            mode: BalancerMode::Equalizer,
        },
        limit: 100,
    };
    assert_eq!(config, expected_config)
}

#[test]
fn invalid_updates_config() {
    let mut app = default_app();
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

    let manager_addr = init_manager(&mut app, instantiate_msg, &attach_funds).unwrap();

    // Unauthorized
    let update_cfg_msg = UpdateConfig {
        owner_id: Some("new_owner".to_string()),
        slot_granularity_time: Some(1234),
        paused: Some(true),
        agent_fee: Some(0),
        gas_base_fee: Some(1111),
        gas_action_fee: Some(2222),
        gas_query_fee: Some(3333),
        gas_wasm_query_fee: Some(4444),
        gas_price: Some(GasPrice {
            numerator: 555,
            denominator: 666,
            gas_adjustment_numerator: 777,
        }),
        min_tasks_per_agent: Some(1),
        agents_eject_threshold: Some(3),
        balancer: Some(RoundRobinBalancer::new(BalancerMode::Equalizer)),
    };
    let err: ContractError = app
        .execute_contract(
            // Not admin
            Addr::unchecked(ANYONE),
            manager_addr.clone(),
            &ExecuteMsg::UpdateConfig(update_cfg_msg),
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ContractError::Unauthorized {});

    // Invalid gas_price
    let update_cfg_msg = UpdateConfig {
        owner_id: Some("new_owner".to_string()),
        slot_granularity_time: Some(1234),
        paused: Some(true),
        agent_fee: Some(0),
        gas_base_fee: Some(1111),
        gas_action_fee: Some(2222),
        gas_query_fee: Some(3333),
        gas_wasm_query_fee: Some(4444),
        gas_price: Some(GasPrice {
            numerator: 555,
            denominator: 0,
            gas_adjustment_numerator: 777,
        }),
        min_tasks_per_agent: Some(1),
        agents_eject_threshold: Some(3),
        balancer: Some(RoundRobinBalancer::new(BalancerMode::Equalizer)),
    };
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            manager_addr.clone(),
            &ExecuteMsg::UpdateConfig(update_cfg_msg),
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ContractError::InvalidGasPrice {});

    // Invalid owner
    let update_cfg_msg = UpdateConfig {
        owner_id: Some("New_owner".to_string()),
        slot_granularity_time: Some(1234),
        paused: Some(true),
        agent_fee: Some(0),
        gas_base_fee: Some(1111),
        gas_action_fee: Some(2222),
        gas_query_fee: Some(3333),
        gas_wasm_query_fee: Some(4444),
        gas_price: Some(GasPrice {
            numerator: 555,
            denominator: 666,
            gas_adjustment_numerator: 777,
        }),
        min_tasks_per_agent: Some(1),
        agents_eject_threshold: Some(3),
        balancer: Some(RoundRobinBalancer::new(BalancerMode::Equalizer)),
    };
    let err: ContractError = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            manager_addr,
            &ExecuteMsg::UpdateConfig(update_cfg_msg),
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

    let instantiate_msg: InstantiateMsg = default_instantiate_message();
    let manager_addr = init_manager(&mut app, instantiate_msg, &coins(100, DENOM)).unwrap();

    let cw20_addr = init_cw20(&mut app);
    app.execute_contract(
        Addr::unchecked(ADMIN),
        cw20_addr.clone(),
        &cw20::Cw20ExecuteMsg::Send {
            contract: manager_addr.to_string(),
            amount: Uint128::new(555),
            msg: to_binary(&ReceiveMsg::RefillCw20Balance {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    let wallet_balances = query_cw20_wallet_manager(&app, &manager_addr, ADMIN);
    assert_eq!(
        wallet_balances,
        vec![Cw20CoinVerified {
            address: cw20_addr.clone(),
            amount: Uint128::new(555),
        }]
    );

    let available_balances = query_manager_balances(&app, &manager_addr);
    assert_eq!(
        available_balances,
        BalancesResponse {
            available_native_balance: coins(100, DENOM),
            available_cw20_balance: vec![Cw20CoinVerified {
                address: cw20_addr,
                amount: Uint128::new(555),
            }],
        }
    )
}

#[test]
fn cw20_bad_messages() {
    let mut app = default_app();

    let instantiate_msg: InstantiateMsg = default_instantiate_message();
    let manager_addr = init_manager(&mut app, instantiate_msg, &[]).unwrap();

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
            "croncat_sdk_core::msg::ManagerReceiveMsg",
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
            "croncat_sdk_core::msg::ManagerReceiveMsg",
            "Expected to parse either a `true`, `false`, or a `null`."
        ))
    );
}

#[test]
fn cw20_withdraws() {
    let mut app = default_app();

    let instantiate_msg: InstantiateMsg = default_instantiate_message();
    let manager_addr = init_manager(&mut app, instantiate_msg, &[]).unwrap();

    // refill balance
    let cw20_addr = init_cw20(&mut app);
    app.execute_contract(
        Addr::unchecked(ADMIN),
        cw20_addr.clone(),
        &cw20::Cw20ExecuteMsg::Send {
            contract: manager_addr.to_string(),
            amount: Uint128::new(1000),
            msg: to_binary(&ReceiveMsg::RefillCw20Balance {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    // Withdraw half
    let user_balance: cw20::BalanceResponse = app
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
        &ExecuteMsg::WithdrawCw20WalletBalances {
            cw20_amounts: vec![Cw20Coin {
                address: cw20_addr.to_string(),
                amount: Uint128::new(500),
            }],
        },
        &[],
    )
    .unwrap();

    // Check it updated on cw20 state
    let new_user_balance: cw20::BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            cw20_addr.clone(),
            &cw20::Cw20QueryMsg::Balance {
                address: ADMIN.to_owned(),
            },
        )
        .unwrap();
    assert_eq!(
        new_user_balance.balance,
        user_balance.balance + Uint128::new(500)
    );

    // Check it updated on manager
    let manager_wallet_balance = query_cw20_wallet_manager(&app, &manager_addr, ADMIN);
    assert_eq!(
        manager_wallet_balance,
        vec![Cw20CoinVerified {
            address: cw20_addr.clone(),
            amount: Uint128::new(500),
        }]
    );

    // Check available got updated too
    let available_balances = query_manager_balances(&app, &manager_addr);
    assert_eq!(
        available_balances.available_cw20_balance,
        vec![Cw20CoinVerified {
            address: cw20_addr.clone(),
            amount: Uint128::new(500),
        }]
    );

    // Withdraw rest
    app.execute_contract(
        Addr::unchecked(ADMIN),
        manager_addr.clone(),
        &ExecuteMsg::WithdrawCw20WalletBalances {
            cw20_amounts: vec![Cw20Coin {
                address: cw20_addr.to_string(),
                amount: Uint128::new(500),
            }],
        },
        &[],
    )
    .unwrap();

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
        user_balance.balance + Uint128::new(1000)
    );

    // Check it updated on manager
    let manager_wallet_balance = query_cw20_wallet_manager(&app, &manager_addr, ADMIN);
    assert_eq!(manager_wallet_balance, vec![]);

    // Check available got updated too
    let available_balances = query_manager_balances(&app, &manager_addr);
    assert_eq!(available_balances.available_cw20_balance, vec![]);
}
