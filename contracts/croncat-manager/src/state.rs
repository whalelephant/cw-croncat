use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use croncat_sdk_core::balancer::RoundRobinBalancer;
use croncat_sdk_core::types::GasPrice;
use cw_storage_plus::{Item, Map};

pub use croncat_sdk_core::types::Config;

pub const CONFIG: Item<Config> = Item::new("config");

// Tasks + rewards balances
/// Available native balance of the contract
/// Key: Denom
/// Value: Amount
pub const AVAILABLE_NATIVE_BALANCE: Map<&str, Uint128> = Map::new("available_native_balance");
/// Available cw20 balance of the contract
/// Key: Cw20 Addr
/// Value: Amount
pub const AVAILABLE_CW20_BALANCE: Map<&Addr, Uint128> = Map::new("available_cw20_balance");

// Accrued Agent reward balance
pub const AGENT_BALANCES_NATIVE: Map<(&Addr, &str), Uint128> = Map::new("agent_balances_native");

pub const AGENT_BALANCES_CW20: Map<(&Addr, &Addr), Uint128> = Map::new("agent_balances_cw20");

pub const USERS_BALANCES_CW20: Map<(&Addr, &Addr), Uint128> = Map::new("users_balances_cw20");
