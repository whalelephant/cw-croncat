use cosmwasm_std::Addr;
use croncat_sdk_factory::msg::ContractMetadata;
use cw_storage_plus::Map;

/// Contract name to the metadata
pub const CONTRACT_METADATAS: Map<&str, ContractMetadata> = Map::new("contract_metadatas");

/// Contract name to the Addr
pub const CONTRACT_ADDRS: Map<&str, Addr> = Map::new("contract_addrs");

/// Temporary Map reply id to label
pub const CONTRACT_LABELS: Map<u64, String> = Map::new("contract_labels");