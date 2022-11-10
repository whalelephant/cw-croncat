use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Account {
    pub name: String,
    pub address: String,
    pub mnemonic: String,
}