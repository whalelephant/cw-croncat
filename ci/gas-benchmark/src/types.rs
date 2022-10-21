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

#[derive(Clone, Debug)]
pub(crate) struct ApproxGasCosts {
    pub gas_per_action: u64,
    pub gas_for_proxy_call: u64,
    pub gas_for_task_unregister: u64,
    pub max_gas_per_action: u64,
}

impl ApproxGasCosts {
    pub(crate) fn approx_base_gas(&self) -> u64 {
        self.gas_for_proxy_call + self.gas_for_task_unregister
    }

    pub(crate) fn approx_gas_per_action(&self) -> u64 {
        self.gas_per_action
    }

    pub(crate) fn approx_gas_for_unregister(&self) -> u64 {
        self.gas_for_task_unregister
    }

    pub(crate) fn max_gas_per_action(&self) -> u64 {
        self.max_gas_per_action
    }
}
