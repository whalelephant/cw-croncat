#[cfg(test)]
mod tests {
    use crate::{
        helpers::validate_transforms,
        msg::InstantiateMsg,
        tests::{
            helpers::{
                default_app, default_instantiate_msg, init_agents, init_cw20, init_factory,
                init_manager, init_mod_balances, init_tasks,
            },
            ANYONE, DENOM, PARTICIPANT1,
        },
    };
    use cosmwasm_std::{coins, to_binary, Addr, BankMsg, Binary, CosmosMsg, WasmMsg, WasmQuery};
    use croncat_mod_balances::msg::QueryMsg as BalancesQueryMsg;
    use croncat_mod_generic::types::{PathToValue, ValueIndex};
    use croncat_sdk_core::types::AmountForOneTask;
    use croncat_sdk_tasks::types::{
        Action, Boundary, BoundaryTime, CosmosQuery, CroncatQuery, Interval, Task, TaskRequest,
        Transform,
    };
    use cw20::Cw20QueryMsg;
    use cw_multi_test::Executor;

    #[test]
    fn test_validate_queries() {
        let mut app = default_app();
        let factory_addr = init_factory(&mut app);

        let _ = init_manager(&mut app, &factory_addr);
        let _ = init_agents(&mut app, &factory_addr);
        let instantiate_msg: InstantiateMsg = default_instantiate_msg();
        let tasks_addr = init_tasks(&mut app, &instantiate_msg, &factory_addr);
        let balances_addr = init_mod_balances(&mut app, &factory_addr);
        let cw20_addr = init_cw20(&mut app);

        let bad_addr = Addr::unchecked("doesnt_exist").to_string();

        let queries = vec![
            CosmosQuery::Croncat(CroncatQuery {
                contract_addr: balances_addr.to_string(),
                msg: to_binary(&BalancesQueryMsg::GetBalance {
                    address: Addr::unchecked(ANYONE).to_string(),
                    denom: DENOM.to_string(),
                })
                .unwrap(),
                check_result: true,
            }),
            CosmosQuery::Wasm(WasmQuery::Smart {
                contract_addr: cw20_addr.to_string(),
                msg: to_binary(&Cw20QueryMsg::TokenInfo {}).unwrap(),
            }),
            CosmosQuery::Wasm(WasmQuery::Raw {
                contract_addr: tasks_addr.to_string(),
                key: Binary::from("config".to_string().into_bytes()),
            }),
            CosmosQuery::Wasm(WasmQuery::ContractInfo {
                contract_addr: tasks_addr.to_string(),
            }),
        ];
        let bad_queries = vec![
            CosmosQuery::Croncat(CroncatQuery {
                contract_addr: balances_addr.to_string(),
                msg: Binary::default(),
                check_result: true,
            }),
            CosmosQuery::Croncat(CroncatQuery {
                contract_addr: bad_addr.clone(),
                msg: Binary::default(),
                check_result: true,
            }),
            CosmosQuery::Wasm(WasmQuery::Smart {
                contract_addr: cw20_addr.to_string(),
                msg: Binary::default(),
            }),
            CosmosQuery::Wasm(WasmQuery::Smart {
                contract_addr: bad_addr.clone(),
                msg: Binary::default(),
            }),
            // NOTE: Based on how the VM works, WasmQuery::Raw actually just returns empty, but is successful. yay! #badNotBad
            // CosmosQuery::Wasm(WasmQuery::Raw {
            //     contract_addr: tasks_addr.to_string(),
            //     key: Binary::default(),
            // }),
            // CosmosQuery::Wasm(WasmQuery::Raw {
            //     contract_addr: bad_addr.clone(),
            //     key: Binary::default(),
            // }),
            CosmosQuery::Wasm(WasmQuery::ContractInfo {
                contract_addr: bad_addr,
            }),
        ];

        // Test scenarios with different query types and combinations.
        let test_scenarios = vec![
            // Test with a single successful query.
            (vec![queries[0].clone()], true),
            // Test with a single failing query.
            (vec![bad_queries[0].clone()], false),
            (vec![bad_queries[1].clone()], false),
            (vec![bad_queries[2].clone()], false),
            (vec![bad_queries[3].clone()], false),
            (vec![bad_queries[4].clone()], false),
            // Test with multiple successful queries.
            (queries.clone(), true),
            // Test with multiple queries, where one fails.
            (vec![queries[0].clone(), bad_queries[0].clone()], false),
            // Test with multiple failing queries.
            (bad_queries, false),
        ];

        for (i, (qs, expected_result)) in test_scenarios.into_iter().enumerate() {
            let task = TaskRequest {
                interval: Interval::Immediate,
                boundary: None,
                stop_on_fail: false,
                actions: vec![Action {
                    msg: BankMsg::Send {
                        to_address: Addr::unchecked(PARTICIPANT1).to_string(),
                        amount: coins(5, DENOM),
                    }
                    .into(),
                    gas_limit: Some(50_000),
                }],
                queries: Some(qs.clone()),
                transforms: None,
                cw20: None,
            };

            let res = app.execute_contract(
                Addr::unchecked(ANYONE),
                tasks_addr.clone(),
                &crate::msg::ExecuteMsg::CreateTask {
                    task: Box::new(task),
                },
                &coins(500_000, DENOM),
            );
            assert_eq!(
                res.is_ok(),
                expected_result,
                "Unexpected result for test scenario {}: {:?}",
                i,
                qs
            );
        }
    }

    #[test]
    fn test_validate_transforms() {
        let action_msg = r#"{"transfer":{"recipient":"cosmosx46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v","amount":"100"}}"#.as_bytes();
        let query_msg =
            r#"{"balance":{"address":"cosmos1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v"}}"#.as_bytes();

        let action = Action {
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "some_contract_addr".to_string(),
                msg: Binary::from(action_msg),
                funds: vec![],
            }),
            gas_limit: None,
        };

        let query = CosmosQuery::Wasm(WasmQuery::Smart {
            contract_addr: "some_contract_addr".to_string(),
            msg: Binary::from(query_msg),
        });

        let query_response_path = PathToValue::from(vec![
            ValueIndex::from("balance".to_string()),
            ValueIndex::from("denom".to_string()),
        ]);

        let action_path = PathToValue::from(vec![
            ValueIndex::from("transfer".to_string()),
            ValueIndex::from("amount".to_string()),
        ]);

        let transform = Transform {
            action_idx: 0,
            query_idx: 0,
            query_response_path,
            action_path,
        };

        let mut task = Task {
            actions: vec![action],
            queries: vec![query],
            transforms: vec![transform],
            owner_addr: Addr::unchecked("owner"),
            interval: Interval::Once,
            boundary: Boundary::Time(BoundaryTime {
                start: None,
                end: None,
            }),
            stop_on_fail: true,
            version: "1.0".to_string(),
            amount_for_one_task: AmountForOneTask::default(),
        };

        assert!(validate_transforms(&task));

        // Test invalid action index
        task.transforms[0].action_idx = 1;
        assert!(!validate_transforms(&task));
        task.transforms[0].action_idx = 0;

        // Test invalid query index
        task.transforms[0].query_idx = 1;
        assert!(!validate_transforms(&task));
        task.transforms[0].query_idx = 0;

        // Test invalid action path
        task.transforms[0]
            .action_path
            .0
            .push(ValueIndex::from("invalid_key".to_string()));
        assert!(!validate_transforms(&task));
    }
}
