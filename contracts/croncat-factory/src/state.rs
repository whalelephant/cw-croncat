use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use croncat_sdk_factory::msg::ContractMetadata;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner_addr: Addr,
}

/// Contract config, just the owner address for now, preferably dao
pub const CONFIG: Item<Config> = Item::new("config");

/// Contract name with the version to the metadata
pub const CONTRACT_METADATAS: Map<(&str, &[u8]), ContractMetadata> = Map::new("contract_metadatas");

/// Contract name with the version to the Addr
pub const CONTRACT_ADDRS: Map<(&str, &[u8]), Addr> = Map::new("contract_addrs");

/// Latest contract name to the Addr
pub const LATEST_ADDRS: Map<&str, Addr> = Map::new("latest_addrs");

// Latest contract name to the version
pub const LATEST_VERSIONS: Map<&str, [u8; 2]> = Map::new("latest_versions");

/// Temporary Map reply id to label
pub const CONTRACT_NAMES: Map<u64, String> = Map::new("contract_names");
