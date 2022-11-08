# Generic query

This CosmWasm based library helps querying contracts and dealing with query results.

You can see how it is used for qeneric query in [rules contract](https://github.com/CronCats/cw-croncat/tree/main/contracts/cw-rules), which is a part of [CronCat contracts](https://github.com/CronCats/cw-croncat).

## GenericQuery structure

The main idea is to provide the structure with both information about query request and anticipated result:
```bash
pub struct GenericQuery {
    pub contract_addr: String,
    pub msg: Binary,
    pub gets: Vec<ValueIndex>,

    pub ordering: ValueOrdering,
    pub value: Binary,
}
```

With `contract_addr` and `msg` one can create a query to the address with this message and parse this binary into json-like result.

`gets` field defines the "path" throgh this result to specific element. For example, if the query result looks like this:
```bash
{
  "members": [
    {
      "addr": "alice",
      "weight": 1
    },
    {
      "addr": "bob",
      "weight": 2
    }
  ]
}
```
then
```bash
let gets = vec![
    ValueIndex::Key("members".to_string()),
    ValueIndex::Index(1),
    ValueIndex::Key("weight".to_string()),
]
```
points to the weight of Bob, which is 2.

`value` field is the amount which will be compared with the query result.

This library also implements `ValueOrd` trait for `Value`.
```bash
pub trait ValueOrd {
    fn lt_g(&self, other: &Self) -> StdResult<bool>;
    fn le_g(&self, other: &Self) -> StdResult<bool>;
    fn bt_g(&self, other: &Self) -> StdResult<bool>;
    fn be_g(&self, other: &Self) -> StdResult<bool>;
    fn equal(&self, other: &Self) -> bool;
}
```
That allows as to compare anticipated and received results (depending on whether they should be equal or one bigger than another).

You can see the usage example in the implementation of [`generic_query`](https://github.com/CronCats/cw-croncat/blob/8c85201856c3dfa89069b5fe97540c3f0d5ee5fa/contracts/cw-rules/src/contract.rs#L239) from CronCat rules contract.
