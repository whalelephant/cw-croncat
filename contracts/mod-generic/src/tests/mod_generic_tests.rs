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

    fn batch_query(queries: Vec<CosmosQuery>) -> Result<Option<QueryResponse>, StdError> {
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

        let result = batch_query(queries)?;
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_batch_query_no_check_result() -> Result<(), Box<dyn Error>> {
        let (app, contract_addr, _, cw20_addr, _) = proper_instantiate();
        let queries = vec![CosmosQuery::Wasm(WasmQuery::Smart {
            contract_addr: cw20_addr.to_string(),
            msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
        })];
        let result: Option<QueryResponse> = app.wrap().query(
            &WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&QueryMsg::BatchQuery { queries })?,
            }
            .into(),
        )?;
        assert!(result.is_some());
        let res = result.unwrap();
        assert!(res.result);
        let token_info: TokenInfoResponse = from_binary(&res.data)?;
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
        let result: Option<QueryResponse> = app.wrap().query(
            &WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&QueryMsg::BatchQuery { queries })?,
            }
            .into(),
        )?;
        assert!(result.is_some());
        let res = result.unwrap();
        assert!(res.result);
        assert_eq!(
            res.data.to_string(),
            "eyJkZW5vbSI6ImF0b20iLCJhbW91bnQiOiIwIn0="
        );

        // attempt a raw state query
        let queries = vec![CosmosQuery::Wasm(WasmQuery::Raw {
            contract_addr: contract_addr.to_string(),
            key: Binary::from("contract_info".to_string().into_bytes()),
        })];
        let result: Option<QueryResponse> = app.wrap().query(
            &WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&QueryMsg::BatchQuery { queries })?,
            }
            .into(),
        )?;
        assert!(result.is_some());
        let res = result.unwrap();
        assert!(res.result);
        assert_eq!(res.data.to_string(), "WzEyMywzNCw5OSwxMTEsMTEwLDExNiwxMTQsOTcsOTksMTE2LDM0LDU4LDM0LDk5LDExNCw5NywxMTYsMTAxLDU4LDk5LDExNCwxMTEsMTEwLDk5LDk3LDExNiw0NSwxMDksMTExLDEwMCw0NSwxMDMsMTAxLDExMCwxMDEsMTE0LDEwNSw5OSwzNCw0NCwzNCwxMTgsMTAxLDExNCwxMTUsMTA1LDExMSwxMTAsMzQsNTgsMzQsNDgsNDYsNDksMzQsMTI1XQ==");

        // attempt a Contract Info query
        let queries = vec![CosmosQuery::Wasm(WasmQuery::ContractInfo {
            contract_addr: contract_addr.to_string(),
        })];
        let result: Option<QueryResponse> = app.wrap().query(
            &WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&QueryMsg::BatchQuery { queries })?,
            }
            .into(),
        )?;
        assert!(result.is_some());
        let res = result.unwrap();
        assert!(res.result);
        assert_eq!(res.data.to_string(), "eyJjb2RlX2lkIjoxLCJjcmVhdG9yIjoiY3JlYXRvciIsImFkbWluIjpudWxsLCJwaW5uZWQiOmZhbHNlLCJpYmNfcG9ydCI6bnVsbH0=");
        let contract_info: ContractInfoResponse = from_binary(&res.data)?;
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
        // let result: Option<QueryResponse> = result_raw?;
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
        let result: Option<QueryResponse> = app.wrap().query(
            &WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&QueryMsg::BatchQuery { queries })?,
            }
            .into(),
        )?;
        assert!(result.is_none());

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
        let result: Option<QueryResponse> = app.wrap().query(
            &WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&QueryMsg::BatchQuery { queries })?,
            }
            .into(),
        )?;
        assert!(result.is_some());
        let res = result.unwrap();
        assert!(res.result);
        assert_eq!(res.data.to_string(), "eyJjb2RlX2lkIjoxLCJjcmVhdG9yIjoiY3JlYXRvciIsImFkbWluIjpudWxsLCJwaW5uZWQiOmZhbHNlLCJpYmNfcG9ydCI6bnVsbH0=");
        let contract_info: ContractInfoResponse = from_binary(&res.data)?;
        assert_eq!(contract_info.code_id, 1);

        Ok(())
    }
}
