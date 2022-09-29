use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Account {
    pub name: String,
    pub address: String,
    pub mnemonic: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct GasInformation {
    pub gas_used: u64,
    pub native_balance_burned: u128,
}
