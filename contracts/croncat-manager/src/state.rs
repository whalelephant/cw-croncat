use cosmwasm_std::{Addr, Uint128};
use croncat_sdk_core::types::TaskBalance;
use cw_storage_plus::{Item, Map};

pub use croncat_sdk_core::types::Config;

pub const CONFIG: Item<Config> = Item::new("config");

// Accrued Treasury reward balance in native coin
pub const TREASURY_BALANCE: Item<Uint128> = Item::new("treasury_balance");

// Accrued Agent reward balance in native coin
pub const AGENT_REWARDS: Map<&Addr, Uint128> = Map::new("agent_rewards");

// Temporary balances of users before task creation.
// Please do not store your coins for any other use.
pub const TEMP_BALANCES_CW20: Map<(&Addr, &Addr), Uint128> = Map::new("temp_balances_cw20");

pub const TASKS_BALANCES: Map<&[u8], TaskBalance> = Map::new("tasks_balances");
