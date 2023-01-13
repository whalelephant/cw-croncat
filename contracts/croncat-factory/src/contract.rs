#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Reply, Response, StdResult, Storage,
    SubMsg, WasmMsg,
};
use croncat_sdk_factory::msg::{
    ContractMetadata, ContractMetadataResponse, EntryResponse, ModuleInstantiateInfo, VersionKind,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    Config, CONFIG, CONTRACT_ADDRS, CONTRACT_METADATAS, CONTRACT_NAMES, LATEST_ADDRS,
    LATEST_VERSIONS,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:croncat-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const CRONCAT_MANAGER_REPLY_ID: u64 = 0;
const CRONCAT_TASKS_REPLY_ID: u64 = 1;
const CRONCAT_AGENTS_REPLY_ID: u64 = 2;

/// Save metadata and generate wasm msg
fn init_save_metadata_generate_wasm_msg(
    storage: &mut dyn Storage,
    init_info: ModuleInstantiateInfo,
    kind: VersionKind,
    factory: &str,
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
    CONTRACT_METADATAS.save(storage, (&init_info.label, &init_info.version), &metadata)?;
    LATEST_VERSIONS.save(storage, &init_info.label, &init_info.version)?;
    let msg = WasmMsg::Instantiate {
        admin: Some(factory.to_owned()),
        code_id: init_info.code_id,
        msg: init_info.msg,
        funds: vec![],
        label: init_info.label,
    };
    Ok(msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let owner_addr = msg
        .owner_addr
        .map(|human| deps.api.addr_validate(&human))
        .transpose()?
        .unwrap_or(info.sender.clone());
    CONFIG.save(deps.storage, &Config { owner_addr })?;

    CONTRACT_NAMES.save(
        deps.storage,
        CRONCAT_MANAGER_REPLY_ID,
        &msg.manager_module_instantiate_info.label,
    )?;
    let manager_wasm = init_save_metadata_generate_wasm_msg(
        deps.storage,
        msg.manager_module_instantiate_info,
        VersionKind::Manager {},
        env.contract.address.as_str(),
    )?;
    let croncat_manager_msg = SubMsg::reply_on_success(manager_wasm, CRONCAT_MANAGER_REPLY_ID);

    CONTRACT_NAMES.save(
        deps.storage,
        CRONCAT_TASKS_REPLY_ID,
        &msg.tasks_module_instantiate_info.label,
    )?;
    let tasks_wasm = init_save_metadata_generate_wasm_msg(
        deps.storage,
        msg.tasks_module_instantiate_info,
        VersionKind::Tasks {},
        env.contract.address.as_str(),
    )?;
    let croncat_tasks_msg = SubMsg::reply_on_success(tasks_wasm, CRONCAT_TASKS_REPLY_ID);

    CONTRACT_NAMES.save(
        deps.storage,
        CRONCAT_AGENTS_REPLY_ID,
        &msg.agents_module_instantiate_info.label,
    )?;
    let agents_wasm = init_save_metadata_generate_wasm_msg(
        deps.storage,
        msg.agents_module_instantiate_info,
        VersionKind::Agents {},
        env.contract.address.as_str(),
    )?;
    let croncat_agents_msg = SubMsg::reply_on_success(agents_wasm, CRONCAT_AGENTS_REPLY_ID);

    let query_modules_msg: Vec<SubMsg> = msg
        .library_modules_instantiate_info
        .into_iter()
        .enumerate()
        .map(|(id, init_info)| {
            let reply_id = id as u64 + 3;
            CONTRACT_NAMES.save(deps.storage, CRONCAT_AGENTS_REPLY_ID, &init_info.label)?;
            let query_wasm = init_save_metadata_generate_wasm_msg(
                deps.storage,
                init_info,
                VersionKind::Library {},
                env.contract.address.as_str(),
            )?;
            Ok(SubMsg::reply_on_success(query_wasm, reply_id))
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("sender", info.sender)
        .add_submessage(croncat_manager_msg)
        .add_submessage(croncat_tasks_msg)
        .add_submessage(croncat_agents_msg)
        .add_submessages(query_modules_msg))
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
        ExecuteMsg::Deploy {
            kind,
            module_instantiate_info,
        } => execute_deploy(deps, env, kind, module_instantiate_info),
        ExecuteMsg::Remove {
            contract_name,
            version,
        } => execute_remove(deps, contract_name, version),
        ExecuteMsg::UpdateMetadataChangelog {
            contract_name,
            version,
            new_changelog,
        } => execute_update_metadata_changelog(deps, contract_name, version, new_changelog),
    }
}

fn execute_update_metadata_changelog(
    deps: DepsMut,
    contract_name: String,
    version: [u8; 2],
    new_changelog: Option<String>,
) -> Result<Response, ContractError> {
    CONTRACT_METADATAS.update(deps.storage, (&contract_name, &version), |metadata_res| {
        match metadata_res {
            Some(mut metadata) => {
                metadata.changelog_url = new_changelog;
                Ok(metadata)
            }
            None => Err(ContractError::UnknownContract {}),
        }
    })?;
    Ok(Response::new().add_attribute("action", "update_metadata_changelog"))
}

fn execute_remove(
    deps: DepsMut,
    contract_name: String,
    version: [u8; 2],
) -> Result<Response, ContractError> {
    // Can't remove latest
    let latest_version = LATEST_VERSIONS
        .may_load(deps.storage, &contract_name)?
        .ok_or(ContractError::UnknownContract {})?;

    if latest_version == version {
        return Err(ContractError::LatestVersionRemove {});
    }
    CONTRACT_METADATAS.remove(deps.storage, (&contract_name, &version));
    CONTRACT_ADDRS.remove(deps.storage, (&contract_name, &version));

    Ok(Response::new().add_attribute("action", "remove"))
}

fn execute_deploy(
    deps: DepsMut,
    env: Env,
    kind: VersionKind,
    module_instantiate_info: ModuleInstantiateInfo,
) -> Result<Response, ContractError> {
    let wasm = init_save_metadata_generate_wasm_msg(
        deps.storage,
        module_instantiate_info,
        kind,
        env.contract.address.as_str(),
    )?;
    let msg = SubMsg::reply_on_success(wasm, 0);

    Ok(Response::new()
        .add_attribute("action", "deploy")
        .add_submessage(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::LatestContracts {} => to_binary(&query_latest_contracts(deps)?),
        QueryMsg::LatestContract { contract_name } => {
            to_binary(&query_latest_contract(deps, contract_name)?)
        }
        QueryMsg::VersionsByContractName { contract_name } => {
            to_binary(&query_versions_by_contract_name(deps, contract_name)?)
        }
        QueryMsg::ContractNames {} => to_binary(&query_contract_names(deps)?),
        QueryMsg::AllEntries {} => to_binary(&query_all_entries(deps)?),
    }
}

fn query_all_entries(deps: Deps) -> StdResult<Vec<EntryResponse>> {
    let metadatas: Vec<((String, Vec<u8>), ContractMetadata)> = CONTRACT_METADATAS
        .range(deps.storage, None, None, Order::Ascending)
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

fn query_contract_names(deps: Deps) -> StdResult<Vec<String>> {
    CONTRACT_ADDRS
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|res| res.map(|(contract_name, _)| contract_name))
        .collect()
}

fn query_versions_by_contract_name(
    deps: Deps,
    contract_name: String,
) -> StdResult<Vec<ContractMetadataResponse>> {
    let metadatas: Vec<(Vec<u8>, ContractMetadata)> = CONTRACT_METADATAS
        .prefix(&contract_name)
        .range(deps.storage, None, None, Order::Ascending)
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
    let msg_id = msg.id;
    let res = parse_reply_instantiate_data(msg)?;

    let contract_name = CONTRACT_NAMES.load(deps.storage, msg_id)?;
    // Not needed anymore
    CONTRACT_NAMES.remove(deps.storage, msg_id);

    let contract_address = deps.api.addr_validate(&res.contract_address)?;
    LATEST_ADDRS.save(deps.storage, &contract_name, &contract_address)?;

    let latest_version = LATEST_VERSIONS.load(deps.storage, &contract_name)?;

    // let metadata = CONTRACT_METADATAS.load(deps.storage, (&contract_name, &latest_version))?;
    CONTRACT_ADDRS.save(
        deps.storage,
        (&contract_name, &latest_version),
        &contract_address,
    )?;

    Ok(Response::new())
}
