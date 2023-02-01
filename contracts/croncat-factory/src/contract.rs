#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order, Reply, Response, StdResult,
    Storage, SubMsg, WasmMsg,
};
use croncat_sdk_factory::msg::{
    ContractMetadata, ContractMetadataResponse, EntryResponse, ModuleInstantiateInfo, VersionKind,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    Config, TempReply, CONFIG, CONTRACT_ADDRS, CONTRACT_METADATAS, LATEST_ADDRS, LATEST_VERSIONS,
    TEMP_REPLY,
};

// version info for migration info
const CONTRACT_NAME: &str = "crate:croncat-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Save metadata and generate wasm msg
/// Note: this will override contract metadata if same contract name and version was stored already
fn init_save_metadata_generate_wasm_msg(
    storage: &mut dyn Storage,
    init_info: ModuleInstantiateInfo,
    kind: VersionKind,
    factory: &str,
    funds: Vec<Coin>,
) -> StdResult<WasmMsg> {
    let metadata = ContractMetadata {
        kind,
        code_id: init_info.code_id,
        version: init_info.version,
        commit_id: init_info.commit_id,
        checksum: init_info.checksum,
        changelog_url: init_info.changelog_url,
        schema: init_info.schema,
    };
    CONTRACT_METADATAS.save(
        storage,
        (&init_info.contract_name, &init_info.version),
        &metadata,
    )?;
    LATEST_VERSIONS.save(storage, &init_info.contract_name, &init_info.version)?;

    let msg = WasmMsg::Instantiate {
        admin: Some(factory.to_owned()),
        code_id: init_info.code_id,
        msg: init_info.msg,
        funds,
        // Formats to `CronCat:manager:0.1`
        label: format!(
            "CronCat:{:?}:{:?}.{:?}",
            init_info.contract_name, init_info.version[0], init_info.version[1]
        ),
    };
    Ok(msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let owner_addr = msg
        .owner_addr
        .map(|human| deps.api.addr_validate(&human))
        .transpose()?
        .unwrap_or_else(|| info.sender.clone());
    CONFIG.save(deps.storage, &Config { owner_addr })?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // Any factory action should be done by the owner_addr
    let config = CONFIG.load(deps.storage)?;
    if config.owner_addr != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    match msg {
        ExecuteMsg::UpdateConfig { owner_addr } => execute_update_config(deps, owner_addr),
        ExecuteMsg::Deploy {
            kind,
            module_instantiate_info,
        } => execute_deploy(deps, env, info, kind, module_instantiate_info),
        ExecuteMsg::Remove {
            contract_name,
            version,
        } => execute_remove(deps, contract_name, version),
        ExecuteMsg::UpdateMetadata {
            contract_name,
            version,
            changelog_url,
            schema,
        } => execute_update_metadata(deps, contract_name, version, changelog_url, schema),
    }
}

fn execute_update_config(deps: DepsMut, owner_addr: String) -> Result<Response, ContractError> {
    let config = Config {
        owner_addr: deps.api.addr_validate(&owner_addr)?,
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("owner_addr", config.owner_addr))
}

fn execute_update_metadata(
    deps: DepsMut,
    contract_name: String,
    version: [u8; 2],
    new_changelog: Option<String>,
    schema: Option<String>,
) -> Result<Response, ContractError> {
    let metadata =
        CONTRACT_METADATAS.update(deps.storage, (&contract_name, &version), |metadata_res| {
            match metadata_res {
                Some(mut metadata) => {
                    // Update only if it contains values
                    // No reason to set it from Some(x) to None
                    if new_changelog.is_some() {
                        metadata.changelog_url = new_changelog;
                    }
                    if schema.is_some() {
                        metadata.schema = schema;
                    }
                    Ok(metadata)
                }
                None => Err(ContractError::UnknownContract {}),
            }
        })?;
    Ok(Response::new()
        .add_attribute("action", "update_metadata")
        .add_attribute("changelog_url", format!("{:?}", metadata.changelog_url))
        .add_attribute("schema", format!("{:?}", metadata.schema)))
}

fn execute_remove(
    deps: DepsMut,
    contract_name: String,
    version: [u8; 2],
) -> Result<Response, ContractError> {
    let latest_version = LATEST_VERSIONS
        .may_load(deps.storage, &contract_name)?
        .ok_or(ContractError::UnknownContract {})?;
    // Can't remove latest
    if latest_version == version {
        return Err(ContractError::LatestVersionRemove {});
    }

    let metadata = CONTRACT_METADATAS.load(deps.storage, (&contract_name, &version))?;

    // Can't remove unpaused contract if not a library
    if metadata.kind != VersionKind::Library {
        // Check if paused
        todo!();
    }

    CONTRACT_METADATAS.remove(deps.storage, (&contract_name, &version));
    CONTRACT_ADDRS.remove(deps.storage, (&contract_name, &version));

    Ok(Response::new().add_attribute("action", "remove"))
}

fn execute_deploy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    kind: VersionKind,
    module_instantiate_info: ModuleInstantiateInfo,
) -> Result<Response, ContractError> {
    let contract_name = module_instantiate_info.contract_name.clone();
    let wasm = init_save_metadata_generate_wasm_msg(
        deps.storage,
        module_instantiate_info,
        kind,
        env.contract.address.as_str(),
        info.funds,
    )?;
    let msg = SubMsg::reply_on_success(wasm, 0);

    // Store temporary data that's needed in the reply
    let temp_reply = TempReply { contract_name };
    TEMP_REPLY.save(deps.storage, &temp_reply)?;

    Ok(Response::new()
        .add_attribute("action", "deploy")
        .add_attribute("version_kind", kind.to_string())
        .add_attribute("contract_name", temp_reply.contract_name)
        .add_submessage(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::LatestContracts {} => to_binary(&query_latest_contracts(deps)?),
        QueryMsg::LatestContract { contract_name } => {
            to_binary(&query_latest_contract(deps, contract_name)?)
        }
        QueryMsg::VersionsByContractName {
            contract_name,
            from_index,
            limit,
        } => to_binary(&query_versions_by_contract_name(
            deps,
            contract_name,
            from_index,
            limit,
        )?),
        QueryMsg::ContractNames { from_index, limit } => {
            to_binary(&query_contract_names(deps, from_index, limit)?)
        }
        QueryMsg::AllEntries { from_index, limit } => {
            to_binary(&query_all_entries(deps, from_index, limit)?)
        }
    }
}

fn query_all_entries(
    deps: Deps,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> StdResult<Vec<EntryResponse>> {
    let from_index = from_index.unwrap_or_default();
    let limit = limit.unwrap_or(100);
    let metadatas: Vec<((String, Vec<u8>), ContractMetadata)> = CONTRACT_METADATAS
        .range(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .collect::<StdResult<_>>()?;

    let mut entries = Vec::with_capacity(metadatas.len());
    for ((contract_name, version), metadata) in metadatas {
        let contract_addr = CONTRACT_ADDRS.load(deps.storage, (&contract_name, &version))?;
        let metadata_response = ContractMetadataResponse {
            kind: metadata.kind,
            code_id: metadata.code_id,
            contract_addr,
            version: metadata.version,
            commit_id: metadata.commit_id,
            checksum: metadata.checksum,
            changelog_url: metadata.changelog_url,
            schema: metadata.schema,
        };
        let entry = EntryResponse {
            contract_name,
            metadata: metadata_response,
        };
        entries.push(entry);
    }
    Ok(entries)
}

fn query_contract_names(
    deps: Deps,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> StdResult<Vec<String>> {
    let from_index = from_index.unwrap_or_default();
    let limit = limit.unwrap_or(100);
    CONTRACT_ADDRS
        .keys(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .map(|res| res.map(|(contract_name, _)| contract_name))
        .collect()
}

fn query_versions_by_contract_name(
    deps: Deps,
    contract_name: String,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> StdResult<Vec<ContractMetadataResponse>> {
    let from_index = from_index.unwrap_or_default();
    let limit = limit.unwrap_or(100);
    let metadatas: Vec<(Vec<u8>, ContractMetadata)> = CONTRACT_METADATAS
        .prefix(&contract_name)
        .range(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .collect::<StdResult<_>>()?;

    let mut versions = Vec::with_capacity(metadatas.len());
    for (version, metadata) in metadatas {
        let contract_addr = CONTRACT_ADDRS.load(deps.storage, (&contract_name, &version))?;
        let metadata_response = ContractMetadataResponse {
            kind: metadata.kind,
            code_id: metadata.code_id,
            contract_addr,
            version: metadata.version,
            commit_id: metadata.commit_id,
            checksum: metadata.checksum,
            changelog_url: metadata.changelog_url,
            schema: metadata.schema,
        };
        versions.push(metadata_response);
    }
    Ok(versions)
}

pub fn query_latest_contracts(deps: Deps) -> StdResult<Vec<EntryResponse>> {
    let latest_versions: Vec<(String, [u8; 2])> = LATEST_VERSIONS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<_>>()?;
    let mut entries = Vec::with_capacity(latest_versions.len());
    for (contract_name, version) in latest_versions {
        let contract_addr = CONTRACT_ADDRS.load(deps.storage, (&contract_name, &version))?;
        let metadata = CONTRACT_METADATAS.load(deps.storage, (&contract_name, &version))?;
        let metadata_response = ContractMetadataResponse {
            kind: metadata.kind,
            code_id: metadata.code_id,
            contract_addr,
            version: metadata.version,
            commit_id: metadata.commit_id,
            checksum: metadata.checksum,
            changelog_url: metadata.changelog_url,
            schema: metadata.schema,
        };
        let entry = EntryResponse {
            contract_name,
            metadata: metadata_response,
        };
        entries.push(entry);
    }
    Ok(entries)
}

pub fn query_latest_contract(
    deps: Deps,
    contract_name: String,
) -> StdResult<Option<ContractMetadataResponse>> {
    let latest_contract_version = LATEST_VERSIONS.may_load(deps.storage, &contract_name)?;
    latest_contract_version
        .map(|version| -> StdResult<_> {
            let contract_addr = CONTRACT_ADDRS.load(deps.storage, (&contract_name, &version))?;
            let metadata = CONTRACT_METADATAS.load(deps.storage, (&contract_name, &version))?;
            Ok(ContractMetadataResponse {
                kind: metadata.kind,
                code_id: metadata.code_id,
                contract_addr,
                version: metadata.version,
                commit_id: metadata.commit_id,
                checksum: metadata.checksum,
                changelog_url: metadata.changelog_url,
                schema: metadata.schema,
            })
        })
        .transpose()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let res = parse_reply_instantiate_data(msg)?;

    let contract_name: String = TEMP_REPLY.load(deps.storage)?.contract_name;

    let contract_address = deps.api.addr_validate(&res.contract_address)?;
    LATEST_ADDRS.save(deps.storage, &contract_name, &contract_address)?;

    let latest_version = LATEST_VERSIONS.load(deps.storage, &contract_name)?;

    CONTRACT_ADDRS.save(
        deps.storage,
        (&contract_name, &latest_version),
        &contract_address,
    )?;

    Ok(Response::new())
}
