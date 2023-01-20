use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum BalanceComparator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[cw_serde]
pub struct HasBalanceComparator {
    pub address: String,
    pub required_balance: cw20::Balance,
    pub comparator: BalanceComparator,
}
