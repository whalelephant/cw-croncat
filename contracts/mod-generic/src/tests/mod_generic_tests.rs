use cosmwasm_std::{to_binary, Binary};
use mod_sdk::types::QueryResponse;

use crate::msg::QueryMsg;
use crate::tests::helpers::proper_instantiate;
use cw20_base::msg::{QueryMsg as Cw20QueryMsg};

// use super::helpers::{ANYONE, NATIVE_DENOM};

#[cfg(test)]
mod tests {
    use cosmwasm_std::StdError;

    use crate::types::{CosmosQuery, CroncatQuery};

    use super::*;
    use std::error::Error;

    fn create_mock_croncat_query(contract_addr: &str, msg: Binary, check_result: bool) -> CosmosQuery {
         CosmosQuery::Croncat(CroncatQuery {
            contract_addr: contract_addr.to_string(),
            msg,
            check_result,
        })
    }

    fn batch_query(queries: Vec<CosmosQuery>) -> Result<Option<QueryResponse>, StdError> {
        let (app, contract_addr, _, _) = proper_instantiate();
        app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::BatchQuery { queries })
    }

    #[test]
    fn test_batch_query_no_queries() -> Result<(), Box<dyn Error>> {
        let queries: Vec<CosmosQuery> = vec![];

        let result = batch_query(queries)?;
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_batch_query_no_check_result() -> Result<(), Box<dyn Error>> {
        let queries = vec![
            create_mock_croncat_query("contract2", to_binary(&Cw20QueryMsg::TokenInfo {})?, false),
        ];

        let result = batch_query(queries)?;
        println!("result {:?}", result);
        // TODO: Check if the result matches the expected last query response.
        // assert_eq!(result, Some(QueryResponse { result: true, data: None }));

        Ok(())
    }

    // #[test]
    // fn test_batch_query_check_result_success() -> Result<(), Box<dyn Error>> {
    //     let queries = vec![
    //         create_mock_croncat_query("contract1", b"msg1", true),
    //         create_mock_croncat_query("contract2", b"msg2", true),
    //     ];

    //     let result = batch_query(queries)?;
    //     // TODO: Check if the result matches the expected last query response.
    //     // assert_eq!(result, Some(/*Expected QueryResponse*/));

    //     Ok(())
    // }

    // #[test]
    // fn test_batch_query_check_result_failure() -> Result<(), Box<dyn Error>> {
    //     let queries = vec![
    //         create_mock_croncat_query("contract1", b"msg1", true),
    //         create_mock_croncat_query("contract2", b"msg2", true),
    //         // This query should fail and make the function return None.
    //         create_mock_croncat_query("contract3", b"msg3", true),
    //     ];

    //     let result = batch_query(queries)?;
    //     assert_eq!(result, None);

    //     Ok(())
    // }
}
