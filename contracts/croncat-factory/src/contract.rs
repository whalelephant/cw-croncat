#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Reply, Response, StdResult,
    Storage, SubMsg, WasmMsg,
};
use croncat_sdk_factory::msg::{ContractMetadata, ModuleInstantiateInfo};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{CONTRACT_ADDRS, CONTRACT_LABELS, CONTRACT_METADATAS};

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
    factory: &str,
) -> StdResult<WasmMsg> {
    let metadata = ContractMetadata {
        code_id: init_info.code_id,
        version: init_info.version,
        commit_id: init_info.commit_id,
        changelog_url: init_info.changelog_url,
        schema: init_info.schema,
    };
    CONTRACT_METADATAS.save(storage, &init_info.label, &metadata)?;

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

    CONTRACT_LABELS.save(
        deps.storage,
        CRONCAT_MANAGER_REPLY_ID,
        &msg.manager_module_instantiate_info.label,
    )?;
    let manager_wasm = init_save_metadata_generate_wasm_msg(
        deps.storage,
        msg.manager_module_instantiate_info,
        env.contract.address.as_str(),
    )?;
    let croncat_manager_msg = SubMsg::reply_on_success(manager_wasm, CRONCAT_MANAGER_REPLY_ID);

    CONTRACT_LABELS.save(
        deps.storage,
        CRONCAT_TASKS_REPLY_ID,
        &msg.tasks_module_instantiate_info.label,
    )?;
    let tasks_wasm = init_save_metadata_generate_wasm_msg(
        deps.storage,
        msg.tasks_module_instantiate_info,
        env.contract.address.as_str(),
    )?;
    let croncat_tasks_msg = SubMsg::reply_on_success(tasks_wasm, CRONCAT_TASKS_REPLY_ID);

    CONTRACT_LABELS.save(
        deps.storage,
        CRONCAT_AGENTS_REPLY_ID,
        &msg.agents_module_instantiate_info.label,
    )?;
    let agents_wasm = init_save_metadata_generate_wasm_msg(
        deps.storage,
        msg.agents_module_instantiate_info,
        env.contract.address.as_str(),
    )?;
    let croncat_agents_msg = SubMsg::reply_on_success(agents_wasm, CRONCAT_AGENTS_REPLY_ID);

    let query_modules_msg: Vec<SubMsg> = msg
        .query_modules_instantiate_info
        .into_iter()
        .enumerate()
        .map(|(id, init_info)| {
            let reply_id = id as u64 + 3;
            CONTRACT_LABELS.save(deps.storage, CRONCAT_AGENTS_REPLY_ID, &init_info.label)?;
            let query_wasm = init_save_metadata_generate_wasm_msg(
                deps.storage,
                init_info,
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
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    todo!();
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractAddr { label } => {
            to_binary(&CONTRACT_ADDRS.may_load(deps.storage, &label)?)
        }
        QueryMsg::ContractMetadata { label } => {
            to_binary(&CONTRACT_METADATAS.may_load(deps.storage, &label)?)
        }
        QueryMsg::ContractAddrs {} => to_binary(
            &CONTRACT_ADDRS
                .range(deps.storage, None, None, Order::Ascending)
                .collect::<StdResult<Vec<(String, Addr)>>>()?,
        ),
        QueryMsg::ContractMetadatas {} => to_binary(
            &CONTRACT_METADATAS
                .range(deps.storage, None, None, Order::Ascending)
                .collect::<StdResult<Vec<(String, ContractMetadata)>>>()?,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let contract_name = CONTRACT_LABELS.load(deps.storage, msg.id)?;
    // Not needed anymore
    CONTRACT_LABELS.remove(deps.storage, msg.id);

    let res = parse_reply_instantiate_data(msg)?;
    let contract_address = deps.api.addr_validate(&res.contract_address)?;
    CONTRACT_ADDRS.save(deps.storage, &contract_name, &contract_address)?;

    Ok(Response::new())
}
