#[cfg(test)]
mod tests {
    use crate::{
        tests::{DENOM, PARTICIPANT1, PARTICIPANT2},
        ContractError,
    };
    use cosmwasm_std::{coins, Addr, BankMsg, Binary, CosmosMsg, WasmMsg};
    use croncat_mod_generic::types::{PathToValue, ValueIndex};
    use croncat_sdk_core::types::AmountForOneTask;
    use croncat_sdk_tasks::types::{
        Action, Boundary, BoundaryTime, CosmosQuery, CroncatQuery, Interval, TaskInfo, Transform,
    };

    use crate::helpers::replace_values;

    fn create_query_response_data(json_str: &str) -> Vec<Option<Binary>> {
        let query_json_value = serde_json::from_str::<serde_json::Value>(json_str).unwrap();
        vec![Some(Binary::from(
            serde_json::to_vec(&query_json_value).unwrap(),
        ))]
    }

    fn get_task() -> TaskInfo {
        TaskInfo {
            owner_addr: Addr::unchecked("owner"),
            interval: Interval::Once,
            boundary: Boundary::Time(BoundaryTime {
                start: None,
                end: None,
            }),
            stop_on_fail: true,
            actions: vec![Action {
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: "some_contract_addr".to_string(),
                    msg: Binary::from(r#"{"action_key": "old_value"}"#.as_bytes()),
                    funds: vec![],
                }),
                gas_limit: None,
            }],
            queries: None,
            transforms: vec![Transform {
                action_idx: 0,
                query_idx: 0,
                action_path: PathToValue::from(vec![ValueIndex::Key("action_key".to_string())]),
                query_response_path: PathToValue::from(vec![ValueIndex::Key(
                    "query_key".to_string(),
                )]),
            }],
            version: "1.0".to_string(),
            amount_for_one_task: AmountForOneTask::default(),
            task_hash: "atom:cc4909816ce7ff69f5804e2416d3c437d7367bc7751596845c658050df7"
                .to_string(),
        }
    }

    #[test]
    fn test_real_world_case() {
        // Create the necessary instances
        let mut task = get_task();

        task.queries = Some(vec![CosmosQuery::Croncat(CroncatQuery {
            contract_addr: "swap_pool".to_string(),
            msg: Binary::from(r#"{"get_price": { "asset_id": "atom" }}"#.as_bytes()),
            check_result: false,
        })]);

        task.actions.clear();
        task.actions.push(Action {
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: Addr::unchecked(PARTICIPANT1).to_string(),
                amount: coins(5, DENOM),
            }),
            gas_limit: None,
        });
        task.actions.push(Action {
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: Addr::unchecked(PARTICIPANT2).to_string(),
                amount: coins(5, DENOM),
            }),
            gas_limit: None,
        });

        // Add a new Transform
        task.transforms.clear();
        task.transforms.push(Transform {
            action_idx: 0,
            query_idx: 0,
            query_response_path: PathToValue::from(vec![
                ValueIndex::Key("asset".to_string()),
                ValueIndex::Key("token_output".to_string()),
            ]),
            action_path: PathToValue::from(vec![
                ValueIndex::Key("bank".to_string()),
                ValueIndex::Key("send".to_string()),
                ValueIndex::Key("amount".to_string()),
                ValueIndex::Index(0),
                ValueIndex::Key("amount".to_string()),
            ]),
        });

        let query_response_data = create_query_response_data(
            r#"{"asset": { "kind": "cw20", "token_output": "1234567890"}}"#,
        );

        // Call the replace_values function
        let result = replace_values(&mut task, query_response_data);

        // Assert that there are no errors
        assert!(result.is_ok());

        // Assert that the values are replaced as expected
        if let CosmosMsg::Bank(BankMsg::Send {
            to_address: _,
            amount,
        }) = &task.actions[0].msg
        {
            assert_eq!(amount, &coins(1234567890, DENOM));
        } else {
            panic!("Unexpected message type");
        }
    }

    #[test]
    fn test_single_action_replace() {
        // Create the necessary instances
        let mut task = get_task();

        // Fill a query, but not really used in this test
        task.queries = Some(vec![CosmosQuery::Croncat(CroncatQuery {
            contract_addr: "swap_pool".to_string(),
            msg: Binary::from(r#"{"get_price": { "asset_id": "atom" }}"#.as_bytes()),
            check_result: false,
        })]);

        let query_response_data = create_query_response_data(r#"{"query_key": "new_value"}"#);

        // Call the replace_values function
        let result = replace_values(&mut task, query_response_data);

        // Assert that there are no errors
        assert!(result.is_ok());

        // Assert that the values are replaced as expected
        if let CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: _,
            msg,
            funds: _,
        }) = &task.actions[0].msg
        {
            let msg_value: serde_json::Value = serde_json::from_slice(&msg.0).unwrap();
            let action_key_value = msg_value.get("action_key").unwrap().as_str().unwrap();
            assert_eq!(action_key_value, "new_value");
        } else {
            panic!("Unexpected message type");
        }
    }

    #[test]
    fn test_multiple_actions_replace() {
        // Test successful value replacement in multiple actions
        let mut task = get_task();
        task.actions.push(Action {
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "some_contract_addr".to_string(),
                msg: Binary::from(r#"{"action_key": "old_value"}"#.as_bytes()),
                funds: vec![],
            }),
            gas_limit: None,
        });

        // Fill a query, but not really used in this test
        task.queries = Some(vec![CosmosQuery::Croncat(CroncatQuery {
            contract_addr: "swap_pool".to_string(),
            msg: Binary::from(r#"{"get_price": { "asset_id": "atom" }}"#.as_bytes()),
            check_result: false,
        })]);

        // Add a new Transform for the second action
        task.transforms.push(Transform {
            action_idx: 1,
            query_idx: 0,
            action_path: PathToValue::from(vec![ValueIndex::Key("action_key".to_string())]),
            query_response_path: PathToValue::from(vec![ValueIndex::Key("query_key".to_string())]),
        });

        let query_response_data = create_query_response_data(r#"{"query_key": "new_value"}"#);

        // Call the replace_values function
        let result = replace_values(&mut task, query_response_data);
        assert!(result.is_ok());

        // Assert that the values are replaced as expected for both actions
        for action in task.actions.iter() {
            if let CosmosMsg::Wasm(WasmMsg::Execute { msg, .. }) = &action.msg {
                let msg_value: serde_json::Value = serde_json::from_slice(&msg.0).unwrap();
                let action_key_value = msg_value.get("action_key").unwrap().as_str().unwrap();
                assert_eq!(action_key_value, "new_value");
            } else {
                panic!("Unexpected message type");
            }
        }
    }

    #[test]
    fn test_multiple_transforms_replace() {
        // Test successful value replacement with multiple transforms
        let mut task = get_task();
        task.actions.push(Action {
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "some_contract_addr".to_string(),
                msg: Binary::from(r#"{"another_key": "another_old_value"}"#.as_bytes()),
                funds: vec![],
            }),
            gas_limit: None,
        });

        // Fill a query, but not really used in this test
        task.queries = Some(vec![CosmosQuery::Croncat(CroncatQuery {
            contract_addr: "swap_pool".to_string(),
            msg: Binary::from(r#"{"get_price": { "asset_id": "atom" }}"#.as_bytes()),
            check_result: false,
        })]);

        // Add a new Transform for the existing action, updating a different key
        task.transforms.push(Transform {
            action_idx: 1,
            query_idx: 0,
            action_path: PathToValue::from(vec![ValueIndex::Key("another_key".to_string())]),
            query_response_path: PathToValue::from(vec![ValueIndex::Key(
                "another_query_key".to_string(),
            )]),
        });

        let query_response_data = create_query_response_data(
            r#"{"query_key": "new_value", "another_query_key": "another_new_value"}"#,
        );

        // Call the replace_values function
        let result = replace_values(&mut task, query_response_data);
        assert!(result.is_ok());

        // Assert that the values are replaced as expected
        if let CosmosMsg::Wasm(WasmMsg::Execute { msg, .. }) = &task.actions[0].msg {
            let msg_value: serde_json::Value = serde_json::from_slice(&msg.0).unwrap();
            let action_key_value = msg_value.get("action_key").unwrap().as_str().unwrap();
            assert_eq!(action_key_value, "new_value");
        } else {
            panic!("Unexpected message type");
        }
        if let CosmosMsg::Wasm(WasmMsg::Execute { msg, .. }) = &task.actions[1].msg {
            let msg_value: serde_json::Value = serde_json::from_slice(&msg.0).unwrap();
            let another_key_value = msg_value.get("another_key").unwrap().as_str().unwrap();
            assert_eq!(another_key_value, "another_new_value");
        } else {
            panic!("Unexpected message type");
        }
    }

    #[test]
    fn test_nested_paths_replace() {
        // Test successful value replacement with nested paths
        let mut task = get_task();

        // Fill a query, but not really used in this test
        task.queries = Some(vec![CosmosQuery::Croncat(CroncatQuery {
            contract_addr: "swap_pool".to_string(),
            msg: Binary::from(r#"{"get_price": { "asset_id": "atom" }}"#.as_bytes()),
            check_result: false,
        })]);

        // Update the Transform with nested paths
        task.transforms[0].action_path =
            PathToValue::from(vec![ValueIndex::Key("action_key".to_string())]);
        task.transforms[0].query_response_path = PathToValue::from(vec![
            ValueIndex::Key("outer_query_key".to_string()),
            ValueIndex::Key("query_key".to_string()),
        ]);

        let query_response_data =
            create_query_response_data(r#"{"outer_query_key": {"query_key": "new_value"}}"#);

        // Call the replace_values function
        let result = replace_values(&mut task, query_response_data);
        assert!(result.is_ok());

        // Assert that the values are replaced as expected
        if let CosmosMsg::Wasm(WasmMsg::Execute { msg, .. }) = &task.actions[0].msg {
            let msg_value: serde_json::Value = serde_json::from_slice(&msg.0).unwrap();
            let action_key_value = msg_value.get("action_key").unwrap().as_str().unwrap();
            assert_eq!(action_key_value, "new_value");
        } else {
            panic!("Unexpected message type");
        }
    }

    #[test]
    fn test_out_of_bounds_action_index() {
        let mut task = get_task();

        // Set an out of bounds action index
        task.transforms[0].action_idx = 1;

        let query_response_data = create_query_response_data(r#"{"query_key": "new_value"}"#);

        // Call the replace_values function
        let result = replace_values(&mut task, query_response_data);

        // Assert that there is an error
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ContractError::TaskInvalidTransform {});
    }

    #[test]
    fn test_out_of_bounds_query_index() {
        let mut task = get_task();

        // Set an out of bounds query index
        task.transforms[0].query_idx = 1;

        let query_response_data = create_query_response_data(r#"{"query_key": "new_value"}"#);

        // Call the replace_values function
        let result = replace_values(&mut task, query_response_data);

        // Assert that there is an error
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ContractError::TaskInvalidTransform {});
    }

    #[test]
    fn test_invalid_action_path() {
        let mut task = get_task();

        // Fill a query, but not really used in this test
        task.queries = Some(vec![CosmosQuery::Croncat(CroncatQuery {
            contract_addr: "swap_pool".to_string(),
            msg: Binary::from(r#"{"get_price": { "asset_id": "atom" }}"#.as_bytes()),
            check_result: false,
        })]);

        // Set an invalid action path
        task.transforms[0].action_path =
            PathToValue::from(vec![ValueIndex::Key("non_existent_key".to_string())]);

        let query_response_data = create_query_response_data(r#"{"query_key": "new_value"}"#);

        // Call the replace_values function
        let result = replace_values(&mut task, query_response_data);

        // Assert that there is an error
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ContractError::Std(cosmwasm_std::StdError::GenericErr {
                msg: "Invalid key for value".to_string()
            })
        );
    }

    #[test]
    fn test_invalid_query_response_path() {
        let mut task = get_task();

        // Fill a query, but not really used in this test
        task.queries = Some(vec![CosmosQuery::Croncat(CroncatQuery {
            contract_addr: "swap_pool".to_string(),
            msg: Binary::from(r#"{"get_price": { "asset_id": "atom" }}"#.as_bytes()),
            check_result: false,
        })]);

        // Set an invalid query response path
        task.transforms[0].query_response_path =
            PathToValue::from(vec![ValueIndex::Key("non_existent_query_key".to_string())]);

        let query_response_data = create_query_response_data(r#"{"query_key": "new_value"}"#);

        // Call the replace_values function
        let result = replace_values(&mut task, query_response_data);

        // Assert that there is an error
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ContractError::Std(cosmwasm_std::StdError::GenericErr {
                msg: "Invalid key for value".to_string()
            })
        );
    }

    #[test]
    fn test_invalid_action_message_type() {
        let mut task = get_task();

        // Fill a query, but not really used in this test
        task.queries = Some(vec![CosmosQuery::Croncat(CroncatQuery {
            contract_addr: "swap_pool".to_string(),
            msg: Binary::from(r#"{"get_price": { "asset_id": "atom" }}"#.as_bytes()),
            check_result: false,
        })]);

        // Set an invalid message type
        task.actions[0].msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: "recipient".to_string(),
            amount: vec![],
        });

        let query_response_data = create_query_response_data(r#"{"query_key": "new_value"}"#);

        // Call the replace_values function
        let result = replace_values(&mut task, query_response_data);

        // Assert that there is an error
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ContractError::Std(cosmwasm_std::StdError::GenericErr {
                msg: "Invalid key for value".to_string()
            })
        );
    }

    #[test]
    fn test_invalid_output_data() {
        let mut task = get_task();

        if let CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: _,
            msg,
            funds: _,
        }) = &mut task.actions[0].msg
        {
            *msg = Binary::from(r#"{"action_key": {"$unknown_key$": "value"}}"#.as_bytes());
        }

        let query_response_data = create_query_response_data(r#"{"query_key": "new_value"}"#);

        // Call the replace_values function
        let result = replace_values(&mut task, query_response_data);

        // Assert that there is an error
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ContractError::TaskInvalidTransform {});
    }

    #[test]
    fn test_no_transforms() {
        let mut task = get_task();

        // Clear the transforms vector
        task.transforms.clear();

        let query_response_data = create_query_response_data(r#"{"query_key": "new_value"}"#);

        // Call the replace_values function
        let result = replace_values(&mut task, query_response_data);

        // Assert that there are no errors
        assert!(result.is_ok());
    }
}
