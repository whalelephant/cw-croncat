#[cfg(test)]
mod tests {
    use crate::{
        helpers::validate_transforms,
    };
    use cosmwasm_std::{Binary, CosmosMsg, WasmMsg, WasmQuery, Addr};
    use croncat_mod_generic::types::{PathToValue, ValueIndex};
    use croncat_sdk_core::types::AmountForOneTask;
    use croncat_sdk_tasks::types::{
        Action, CosmosQuery, Transform, Task, Interval, Boundary, BoundaryTime,
    };

    #[test]
    fn test_validate_transforms() {
        let action_msg = r#"{"transfer":{"recipient":"cosmosx46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v","amount":"100"}}"#.as_bytes();
        let query_msg = r#"{"balance":{"address":"cosmos1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v"}}"#.as_bytes();

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
