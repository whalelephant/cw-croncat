# CwRules core

This CosmWasm based library provides types and messages for [rules contract](https://github.com/CronCats/cw-croncat/tree/main/contracts/cw-rules). It is used in both [CronCat Manager contract](https://github.com/CronCats/cw-croncat) and [CronCat Agent](https://github.com/CronCats/croncat-rs) for checking the status of tasks with rules. Tasks with rules are executed only if all rules succeed, hence agents and manager contract must query rules contract.

`QueryMsg` defines several option for rules conditions. 

`RuleResponse<T> = (bool, T)` allows to return boolean result for the query together with some optional specification about the rule failure/success.

## Queries

`GetBalance` queries balance of the address.

`GetCw20Balance` queries cw20 balance of the address for the specified `cw20_contract`.

`HasBalanceGte` checks whether the address has at least `required_balance` (might be both native and cw20)

`CheckOwnerOfNft` checks whether the address owns the NFT.

`CheckProposalStatus` checks whether DAO DAO proposal has passed.

`QueryConstruct` checks a vector of rules, in case of failure returns `false` and the index of the failed rule.

`GenericQuery` is used for creating queries with generic rules, see [`generic-query`](https://github.com/CronCats/cw-croncat/tree/main/packages/generic-query) crate for details.