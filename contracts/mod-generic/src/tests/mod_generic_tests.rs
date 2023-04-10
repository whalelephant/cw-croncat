use cosmwasm_std::{to_binary, Addr, Binary};
use mod_sdk::types::QueryResponse;

use crate::msg::QueryMsg;
use crate::tests::helpers::proper_instantiate;
use croncat_mod_balances::msg::QueryMsg as BalancesQueryMsg;
use cw20_base::msg::QueryMsg as Cw20QueryMsg;

use super::helpers::{ANYONE, NATIVE_DENOM};

#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, from_binary, ContractInfoResponse, StdError, WasmQuery};
    use croncat_mod_balances::types::{BalanceComparator, HasBalanceComparator};
    use cw20::{Balance, TokenInfoResponse};
    use cw_utils::NativeBalance;

    use crate::types::{CosmosQuery, CroncatQuery};

    use super::*;
    use std::error::Error;

    fn batch_query(queries: Vec<CosmosQuery>) -> Result<QueryResponse, StdError> {
        let (app, contract_addr, _, _, _) = proper_instantiate();
        app.wrap().query(
            &WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&QueryMsg::BatchQuery { queries })?,
            }
            .into(),
        )
    }

    #[test]
    fn test_batch_query_no_queries() -> Result<(), Box<dyn Error>> {
        let queries: Vec<CosmosQuery> = vec![];
        let result = batch_query(queries.clone())?;
        let responses: Vec<Option<Binary>> = Vec::with_capacity(queries.len());
        assert_eq!(
            result,
            QueryResponse {
                result: true,
                data: to_binary(&responses)?
            }
        );

        Ok(())
    }

    #[test]
    fn test_batch_query_no_check_result() -> Result<(), Box<dyn Error>> {
        let (app, contract_addr, _, cw20_addr, _) = proper_instantiate();
        let queries = vec![CosmosQuery::Wasm(WasmQuery::Smart {
            contract_addr: cw20_addr.to_string(),
            msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
        })];
        let res: QueryResponse = app.wrap().query(
            &WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&QueryMsg::BatchQuery { queries })?,
            }
            .into(),
        )?;

        assert!(res.result);
        assert_eq!(res.data.to_string(), "WyJleUprWldOcGJXRnNjeUk2Tml3aWJtRnRaU0k2SW5SbGMzUWlMQ0p6ZVcxaWIyd2lPaUpvWld4c2J5SXNJblJ2ZEdGc1gzTjFjSEJzZVNJNklqSXdNaklpZlE9PSJd");
        let bin_responses: Vec<Option<Binary>> = from_binary(&res.data)?;
        assert_eq!(bin_responses[0].as_ref().unwrap().to_string(), "eyJkZWNpbWFscyI6NiwibmFtZSI6InRlc3QiLCJzeW1ib2wiOiJoZWxsbyIsInRvdGFsX3N1cHBseSI6IjIwMjIifQ==".to_string());
        let token_info: TokenInfoResponse = from_binary(bin_responses[0].as_ref().unwrap())?;
        assert_eq!(token_info.name, "test");

        Ok(())
    }

    #[test]
    fn test_batch_query_check_result_success() -> Result<(), Box<dyn Error>> {
        let (app, contract_addr, _, _, balances_addr) = proper_instantiate();
        let queries = vec![CosmosQuery::Croncat(CroncatQuery {
            contract_addr: balances_addr.to_string(),
            msg: to_binary(&BalancesQueryMsg::GetBalance {
                address: Addr::unchecked(ANYONE).to_string(),
                denom: NATIVE_DENOM.to_string(),
            })?,
            check_result: true,
        })];
        let res: QueryResponse = app.wrap().query(
            &WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&QueryMsg::BatchQuery { queries })?,
            }
            .into(),
        )?;
        assert!(res.result);
        assert_eq!(
            res.data.to_string(),
            "WyJleUprWlc1dmJTSTZJbUYwYjIwaUxDSmhiVzkxYm5RaU9pSXdJbjA9Il0="
        );

        // attempt a raw state query
        let queries = vec![CosmosQuery::Wasm(WasmQuery::Raw {
            contract_addr: contract_addr.to_string(),
            key: Binary::from("contract_info".to_string().into_bytes()),
        })];
        let res: QueryResponse = app.wrap().query(
            &WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&QueryMsg::BatchQuery { queries })?,
            }
            .into(),
        )?;
        assert!(res.result);
        assert_eq!(res.data.to_string(), "WyJXekV5TXl3ek5DdzVPU3d4TVRFc01URXdMREV4Tml3eE1UUXNPVGNzT1Rrc01URTJMRE0wTERVNExETTBMRGs1TERFeE5DdzVOeXd4TVRZc01UQXhMRFU0TERrNUxERXhOQ3d4TVRFc01URXdMRGs1TERrM0xERXhOaXcwTlN3eE1Ea3NNVEV4TERFd01DdzBOU3d4TURNc01UQXhMREV4TUN3eE1ERXNNVEUwTERFd05TdzVPU3d6TkN3ME5Dd3pOQ3d4TVRnc01UQXhMREV4TkN3eE1UVXNNVEExTERFeE1Td3hNVEFzTXpRc05UZ3NNelFzTkRnc05EWXNORGtzTXpRc01USTFYUT09Il0=");

        // attempt a Contract Info query
        let queries = vec![CosmosQuery::Wasm(WasmQuery::ContractInfo {
            contract_addr: contract_addr.to_string(),
        })];
        let res: QueryResponse = app.wrap().query(
            &WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&QueryMsg::BatchQuery { queries })?,
            }
            .into(),
        )?;
        assert!(res.result);
        assert_eq!(res.data.to_string(), "WyJleUpqYjJSbFgybGtJam94TENKamNtVmhkRzl5SWpvaVkzSmxZWFJ2Y2lJc0ltRmtiV2x1SWpwdWRXeHNMQ0p3YVc1dVpXUWlPbVpoYkhObExDSnBZbU5mY0c5eWRDSTZiblZzYkgwPSJd");
        let bin_responses: Vec<Option<Binary>> = from_binary(&res.data)?;
        assert_eq!(bin_responses[0].as_ref().unwrap().to_string(), "eyJjb2RlX2lkIjoxLCJjcmVhdG9yIjoiY3JlYXRvciIsImFkbWluIjpudWxsLCJwaW5uZWQiOmZhbHNlLCJpYmNfcG9ydCI6bnVsbH0=".to_string());
        let contract_info: ContractInfoResponse = from_binary(bin_responses[0].as_ref().unwrap())?;
        assert_eq!(contract_info.code_id, 1);

        // NOTE: Needs further support once features = ["cosmwasm_1_2"] is fully ready
        // // attempt a Code Info query
        // let queries = vec![
        //     CosmosQuery::Wasm(WasmQuery::CodeInfo {
        //         code_id: 1,
        //     })
        // ];
        // let result_raw = app.wrap().query(
        //     &WasmQuery::Smart {
        //         contract_addr: contract_addr.clone().to_string(),
        //         msg: to_binary(&QueryMsg::BatchQuery { queries })?,
        //     }
        //     .into(),
        // );
        // let result: QueryResponse = result_raw?;
        // assert!(result.is_some());
        // let res = result.unwrap();
        // assert!(res.result);
        // // assert_eq!(res.data.to_string(), "eyJjb2RlX2lkIjoxLCJjcmVhdG9yIjoiY3JlYXRvciIsImFkbWluIjpudWxsLCJwaW5uZWQiOmZhbHNlLCJpYmNfcG9ydCI6bnVsbH0=");
        // let code_info: CodeInfoResponse = from_binary(&res.data)?;
        // assert_eq!(code_info.code_id, 1);
        // assert_eq!(code_info.creator, Addr::unchecked(CREATOR_ADDR).to_string());

        Ok(())
    }

    #[test]
    fn test_batch_query_check_result_failure() -> Result<(), Box<dyn Error>> {
        let (app, contract_addr, _, _, balances_addr) = proper_instantiate();
        let queries = vec![CosmosQuery::Croncat(CroncatQuery {
            contract_addr: balances_addr.to_string(),
            msg: to_binary(&BalancesQueryMsg::HasBalanceComparator(
                HasBalanceComparator {
                    address: ANYONE.to_string(),
                    required_balance: Balance::Native(NativeBalance(coins(
                        900_000u128,
                        NATIVE_DENOM.to_string(),
                    ))),
                    comparator: BalanceComparator::Gte,
                },
            ))?,
            check_result: true,
        })];
        let res: QueryResponse = app.wrap().query(
            &WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&QueryMsg::BatchQuery { queries })?,
            }
            .into(),
        )?;
        assert!(!res.result);

        Ok(())
    }

    #[test]
    fn test_batch_query_check_multiple() -> Result<(), Box<dyn Error>> {
        let (app, contract_addr, _, cw20_addr, balances_addr) = proper_instantiate();
        let queries = vec![
            CosmosQuery::Croncat(CroncatQuery {
                contract_addr: balances_addr.to_string(),
                msg: to_binary(&BalancesQueryMsg::HasBalanceComparator(
                    HasBalanceComparator {
                        address: ANYONE.to_string(),
                        required_balance: Balance::Native(NativeBalance(coins(
                            0u128,
                            NATIVE_DENOM.to_string(),
                        ))),
                        comparator: BalanceComparator::Gte,
                    },
                ))?,
                check_result: true,
            }),
            CosmosQuery::Croncat(CroncatQuery {
                contract_addr: balances_addr.to_string(),
                msg: to_binary(&BalancesQueryMsg::GetBalance {
                    address: Addr::unchecked(ANYONE).to_string(),
                    denom: NATIVE_DENOM.to_string(),
                })?,
                check_result: true,
            }),
            CosmosQuery::Wasm(WasmQuery::Smart {
                contract_addr: cw20_addr.to_string(),
                msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
            }),
            CosmosQuery::Wasm(WasmQuery::Raw {
                contract_addr: contract_addr.to_string(),
                key: Binary::from("contract_info".to_string().into_bytes()),
            }),
            CosmosQuery::Wasm(WasmQuery::ContractInfo {
                contract_addr: contract_addr.to_string(),
            }),
        ];
        let res: QueryResponse = app.wrap().query(
            &WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&QueryMsg::BatchQuery { queries })?,
            }
            .into(),
        )?;
        assert!(res.result);
        // Checks the whole array response
        assert_eq!(res.data.to_string(), "WyJleUprWlc1dmJTSTZJbUYwYjIwaUxDSmhiVzkxYm5RaU9pSXdJbjA9IiwiZXlKa1pXNXZiU0k2SW1GMGIyMGlMQ0poYlc5MWJuUWlPaUl3SW4wPSIsImV5SmtaV05wYldGc2N5STZOaXdpYm1GdFpTSTZJblJsYzNRaUxDSnplVzFpYjJ3aU9pSm9aV3hzYnlJc0luUnZkR0ZzWDNOMWNIQnNlU0k2SWpJd01qSWlmUT09IiwiV3pFeU15d3pOQ3c1T1N3eE1URXNNVEV3TERFeE5pd3hNVFFzT1Rjc09Ua3NNVEUyTERNMExEVTRMRE0wTERrNUxERXhOQ3c1Tnl3eE1UWXNNVEF4TERVNExEazVMREV4TkN3eE1URXNNVEV3TERrNUxEazNMREV4Tml3ME5Td3hNRGtzTVRFeExERXdNQ3cwTlN3eE1ETXNNVEF4TERFeE1Dd3hNREVzTVRFMExERXdOU3c1T1N3ek5DdzBOQ3d6TkN3eE1UZ3NNVEF4TERFeE5Dd3hNVFVzTVRBMUxERXhNU3d4TVRBc016UXNOVGdzTXpRc05EZ3NORFlzTkRrc016UXNNVEkxWFE9PSIsImV5SmpiMlJsWDJsa0lqb3hMQ0pqY21WaGRHOXlJam9pWTNKbFlYUnZjaUlzSW1Ga2JXbHVJanB1ZFd4c0xDSndhVzV1WldRaU9tWmhiSE5sTENKcFltTmZjRzl5ZENJNmJuVnNiSDA9Il0=");
        let bin_responses: Vec<Option<Binary>> = from_binary(&res.data)?;
        assert_eq!(
            bin_responses[0].as_ref().unwrap().to_string(),
            "eyJkZW5vbSI6ImF0b20iLCJhbW91bnQiOiIwIn0=".to_string()
        );
        let contract_info: ContractInfoResponse = from_binary(bin_responses[4].as_ref().unwrap())?;
        assert_eq!(contract_info.code_id, 1);

        let queries = vec![
            CosmosQuery::Croncat(CroncatQuery {
                contract_addr: balances_addr.to_string(),
                msg: to_binary(&BalancesQueryMsg::GetBalance {
                    address: Addr::unchecked(ANYONE).to_string(),
                    denom: NATIVE_DENOM.to_string(),
                })?,
                check_result: true,
            }),
            // Now this should resolve to FALSE
            CosmosQuery::Croncat(CroncatQuery {
                contract_addr: balances_addr.to_string(),
                msg: to_binary(&BalancesQueryMsg::HasBalanceComparator(
                    HasBalanceComparator {
                        address: ANYONE.to_string(),
                        required_balance: Balance::Native(NativeBalance(coins(
                            10_000u128,
                            NATIVE_DENOM.to_string(),
                        ))),
                        comparator: BalanceComparator::Gte,
                    },
                ))?,
                check_result: true,
            }),
            CosmosQuery::Wasm(WasmQuery::Smart {
                contract_addr: cw20_addr.to_string(),
                msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
            }),
            CosmosQuery::Wasm(WasmQuery::Raw {
                contract_addr: contract_addr.to_string(),
                key: Binary::from("contract_info".to_string().into_bytes()),
            }),
            CosmosQuery::Wasm(WasmQuery::ContractInfo {
                contract_addr: contract_addr.to_string(),
            }),
        ];
        let res: QueryResponse = app.wrap().query(
            &WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&QueryMsg::BatchQuery { queries })?,
            }
            .into(),
        )?;
        assert!(!res.result);

        Ok(())
    }
}
