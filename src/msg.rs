use crate::state::GenericBalance;
use cosmwasm_std::{Addr, Coin};
use cw20::Balance;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner_id: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateSettings {
        owner_id: Option<Addr>,
        slot_granularity: Option<u64>,
        paused: Option<bool>,
        agent_fee: Option<Coin>,
        gas_price: Option<u32>,
        proxy_callback_gas: Option<u32>,
        agent_task_ratio: Option<Vec<u64>>,
        agents_eject_threshold: Option<u64>,
        // treasury_id: Option<Addr>,
    },
    MoveBalances {
        balances: Vec<Balance>,
        account_id: Addr,
    },
    RegisterAgent {
        payable_account_id: Option<Addr>,
    },
    UpdateAgent {
        payable_account_id: Addr,
    },
    CheckInAgent {},
    UnregisterAgent {},
    WithdrawReward {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetBalances {},
    GetAgent { account_id: Addr },
    GetAgentIds {},
    GetAgentTasks { account_id: Addr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub paused: bool,
    pub owner_id: Addr,
    // pub treasury_id: Option<Addr>,
    pub agent_task_ratio: [u64; 2],
    pub agent_active_index: u64,
    pub agents_eject_threshold: u64,
    pub agent_fee: Coin,
    pub gas_price: u32,
    pub proxy_callback_gas: u32,
    pub slot_granularity: u64,
    pub native_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BalancesResponse {
    pub native_denom: String,
    pub available_balance: GenericBalance,
    pub staked_balance: GenericBalance,
    pub cw20_whitelist: Vec<Addr>,
}
