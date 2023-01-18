use std::fmt;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary};

#[cw_serde]
pub struct Config {
    pub owner_addr: Addr,
}

#[cw_serde]
pub struct FactoryInstantiateMsg {
    pub owner_addr: Option<String>,
}

#[cw_serde]
pub enum FactoryExecuteMsg {
    UpdateConfig {
        owner_addr: String,
    },

    Deploy {
        kind: VersionKind,
        module_instantiate_info: ModuleInstantiateInfo,
    },

    Remove {
        contract_name: String,
        version: [u8; 2],
    },

    UpdateMetadata {
        contract_name: String,
        version: [u8; 2],
        changelog_url: Option<String>,
        schema: Option<String>,
    },
}
// TODO: migrate

#[cw_serde]
#[derive(QueryResponses)]
pub enum FactoryQueryMsg {
    #[returns[Config]]
    Config {},

    #[returns[Vec<EntryResponse>]]
    LatestContracts {},

    #[returns[Option<ContractMetadataResponse>]]
    LatestContract { contract_name: String },

    #[returns[Vec<ContractMetadataResponse>]]
    VersionsByContractName {
        contract_name: String,
        from_index: Option<u64>,
        limit: Option<u64>,
    },

    #[returns[Vec<String>]]
    ContractNames {
        from_index: Option<u64>,
        limit: Option<u64>,
    },

    #[returns[Vec<EntryResponse>]]
    AllEntries {
        from_index: Option<u64>,
        limit: Option<u64>,
    },
}

#[cw_serde]
pub struct ContractMetadataResponse {
    pub kind: VersionKind,
    pub code_id: u64,
    pub contract_addr: Addr,
    pub version: [u8; 2],
    pub commit_id: String,
    pub checksum: String,
    pub changelog_url: Option<String>,
    pub schema: Option<String>,
}

#[cw_serde]
pub struct EntryResponse {
    pub contract_name: String,
    pub metadata: ContractMetadataResponse,
}

#[cw_serde]
pub struct ContractMetadata {
    pub kind: VersionKind,
    /// Code ID of the contract to be instantiated.
    pub code_id: u64,

    /// Truncated semver so contracts could programmatically check backward compat
    pub version: [u8; 2],

    /// git commit hash
    pub commit_id: String,

    /// proof of deployed code
    pub checksum: String,

    /// public link to a README about this version
    pub changelog_url: Option<String>,

    /// types/schema - helps keep UI/clients backward compatible
    pub schema: Option<String>,
}

#[cw_serde]
#[derive(Copy)]
pub enum VersionKind {
    Library,
    Manager,
    Tasks,
    Agents,
    // Recipes?
}

impl fmt::Display for VersionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionKind::Library => write!(f, "library"),
            VersionKind::Manager => write!(f, "manager"),
            VersionKind::Tasks => write!(f, "tasks"),
            VersionKind::Agents => write!(f, "agents"),
        }
    }
}

// Reference: https://github.com/DA0-DA0/dao-contracts/blob/fa567797e2f42e70296a2d6f889f341ff80f0695/packages/dao-interface/src/lib.rs#L17
/// Information about the CosmWasm level admin of a contract. Used in
/// conjunction with `ModuleInstantiateInfo` to instantiate modules.
#[cw_serde]
pub enum Admin {
    /// Set the admin to a specified address.
    Address { addr: String },
    /// Sets the admin as the core module address.
    CoreModule {},
}

/// Information needed to instantiate a module.
#[cw_serde]
pub struct ModuleInstantiateInfo {
    /// Code ID of the contract to be instantiated.
    pub code_id: u64,

    /// Truncated semver so contracts could programmatically check backward compat
    pub version: [u8; 2],

    /// git commit hash
    pub commit_id: String,

    /// proof of deployed code
    pub checksum: String,

    /// public link to a README about this version
    pub changelog_url: Option<String>,

    /// types/schema - helps keep UI/clients backward compatible
    pub schema: Option<String>,

    /// Instantiate message to be used to create the contract.
    pub msg: Binary,

    /// Contract name for the instantiated contract.
    pub contract_name: String,
}
