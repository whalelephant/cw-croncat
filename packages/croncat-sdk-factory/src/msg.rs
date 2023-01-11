use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary};

#[cw_serde]
pub struct FactoryInstantiateMsg {
    pub manager_module_instantiate_info: ModuleInstantiateInfo,
    pub tasks_module_instantiate_info: ModuleInstantiateInfo,
    pub agents_module_instantiate_info: ModuleInstantiateInfo,

    pub query_modules_instantiate_info: Vec<ModuleInstantiateInfo>,
}

#[cw_serde]
pub struct FactoryExecuteMsg {

}

#[cw_serde]
#[derive(QueryResponses)]
pub enum FactoryQueryMsg {
    #[returns(Addr)]
    ContractAddr { label: String },
    #[returns(ContractMetadata)]
    ContractMetadata { label: String },
    #[returns(Vec<(String,Addr)>)]
    ContractAddrs {},
    #[returns[Vec<(String,ContractMetadata)>]]
    ContractMetadatas {},
}

#[cw_serde]
pub struct ContractMetadata {
    /// Code ID of the contract to be instantiated.
    pub code_id: u64,

    /// Truncated semver so contracts could programmatically check backward compat
    pub version: [u8; 2],

    /// git commit hash
    pub commit_id: String,

    /// public link to a README about this version
    pub changelog_url: Option<String>,

    /// types/schema - helps keep UI/clients backward compatible
    pub schema: String,
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

    /// public link to a README about this version
    pub changelog_url: Option<String>,

    /// types/schema - helps keep UI/clients backward compatible
    pub schema: String,

    /// Instantiate message to be used to create the contract.
    pub msg: Binary,
    /// Label for the instantiated contract.
    pub label: String,
}
