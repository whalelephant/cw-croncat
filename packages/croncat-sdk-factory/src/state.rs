use cosmwasm_std::Addr;
use cw_storage_plus::Map;

/// Safe way to export map of the croncat-factory, but avoid any contract imports
/// Contract name with the version to the Addr
pub const CONTRACT_ADDRS: Map<(&str, &[u8]), Addr> = Map::new("contract_addrs");
