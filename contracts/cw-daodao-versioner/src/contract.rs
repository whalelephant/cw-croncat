#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw_croncat_core::msg::TaskRequest;
use cw_croncat_core::types::{Action, Interval};

use crate::error::ContractError;
use crate::msg::dao_registry::query::*;
use crate::msg::*;
use crate::state::{CRONCAT_ADDR, REGISTRAR_ADDR, VERSION_MAP};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-daodao-versioner";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let registrar_addr = deps.api.addr_validate(&_msg.registrar_addr)?;
    let croncat_addr = deps.api.addr_validate(&_msg.croncat_addr)?;

    REGISTRAR_ADDR.save(deps.storage, &registrar_addr)?;
    CRONCAT_ADDR.save(deps.storage, &registrar_addr)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::QueryResult {} => query_result(deps, info),
        ExecuteMsg::CreateContractVersioner { name, chain_id } => {
            create_contract_versioner(deps, name, chain_id)
        }
        ExecuteMsg::RemoveContractVersioner { name, chain_id } => {
            remove_contract_versioner(deps, name, chain_id)
        }
        ExecuteMsg::UpdateVersioniser { name, chain_id } => update_versioner(deps, name, chain_id),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VerifyNewVersionAvailable { name, chain_id } => {
            to_binary(&query_new_version_available(deps, name, chain_id)?)
        }
    }
}

pub fn query_result(_deps: DepsMut, _info: MessageInfo) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "query_result"))
}

fn query_registrations(
    deps: Deps,
    registrar_addr: String,
    name: String,
    chain_id: String,
) -> StdResult<ListRegistrationsResponse> {
    let registrar_address = deps.api.addr_validate(&registrar_addr)?;
    let res: ListRegistrationsResponse = deps.querier.query_wasm_smart(
        registrar_address,
        &RegistryQueryMsg::ListRegistrations { name, chain_id },
    )?;
    Ok(res)
}
fn _query_code_id_info(
    deps: Deps,
    registrar_addr: String,
    chain_id: String,
    code_id: u64,
) -> StdResult<GetRegistrationResponse> {
    let registrar_address = deps.api.addr_validate(&registrar_addr)?;
    let res: GetRegistrationResponse = deps.querier.query_wasm_smart(
        registrar_address,
        &RegistryQueryMsg::GetCodeIdInfo { chain_id, code_id },
    )?;
    Ok(res)
}
fn _query_registration(
    deps: Deps,
    registrar_addr: String,
    name: String,
    chain_id: String,
    version: Option<String>,
) -> StdResult<GetRegistrationResponse> {
    let registrar_address = deps.api.addr_validate(&registrar_addr)?;
    let res: GetRegistrationResponse = deps.querier.query_wasm_smart(
        registrar_address,
        &RegistryQueryMsg::GetRegistration {
            name,
            chain_id,
            version,
        },
    )?;
    Ok(res)
}
fn create_contract_versioner(
    deps: DepsMut,
    name: String,
    chain_id: String,
) -> Result<Response, ContractError> {
    if VERSION_MAP
        .may_load(deps.storage, (&name, &chain_id))?
        .is_some()
    {
        return Err(ContractError::ContractAlreadyRegistered(name, chain_id));
    }
    let registrar_addr = REGISTRAR_ADDR.load(deps.storage)?;
    let regs = query_registrations(
        deps.as_ref(),
        registrar_addr.to_string(),
        name.clone(),
        chain_id.clone(),
    )?;
    let registration = regs.registrations.last().unwrap();
    VERSION_MAP.save(
        deps.storage,
        (&registration.contract_name, &chain_id),
        &registration.version,
    )?;

    Ok(Response::new()
        .add_attribute("action", "create_contract_versioner")
        .add_attribute("contract_name", name)
        .add_attribute("chain_id", chain_id))
}

fn remove_contract_versioner(
    deps: DepsMut,
    name: String,
    chain_id: String,
) -> Result<Response, ContractError> {
    if VERSION_MAP
        .may_load(deps.storage, (&name, &chain_id))?
        .is_none()
    {
        return Err(ContractError::ContractNotRegistered(name, chain_id));
    }

    VERSION_MAP.remove(deps.storage, (&name, &chain_id));

    Ok(Response::new()
        .add_attribute("action", "remove_contract_versioner")
        .add_attribute("contract_name", name)
        .add_attribute("chain_id", chain_id))
}

fn is_new_version_available(deps: Deps, name: String, chain_id: String) -> bool {
    let registrar_addr = REGISTRAR_ADDR.load(deps.storage).unwrap();
    let regs = query_registrations(
        deps,
        registrar_addr.to_string(),
        name.clone(),
        chain_id.clone(),
    )
    .unwrap();
    let last = regs.registrations.last().unwrap();
    let current_version = VERSION_MAP.may_load(deps.storage, (&name, &chain_id));

    last.version > current_version.unwrap().unwrap()
}
fn query_new_version_available(deps: Deps, name: String, chain_id: String) -> StdResult<bool> {
    Ok(is_new_version_available(deps, name, chain_id))
}

fn update_versioner(
    deps: DepsMut,
    _env: Env,
    name: String,
    chain_id: String,
) -> Result<Response, ContractError> {
    if is_new_version_available(deps.as_ref(), name, chain_id) {
        return create_versioner_cron_task(deps, _env, name, chain_id);
    }
    Ok(Response::new().add_attribute("action", "update_versioner"))
}
fn create_versioner_cron_task(
    deps: DepsMut,
    _env: Env,
    name: String,
    chain_id: String,
) -> Result<Response, ContractError> {
    let croncat_addr = CRONCAT_ADDR.load(deps.storage)?;
    let cron_name = format!("{name}{chain_id}");
    let action = Action {
        msg: CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: _env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::UpdateVersioniser { name, chain_id })?,
            funds: vec![],
        }),
        gas_limit: None,
    };

    let task_request = TaskRequest {
        interval: Interval::Cron(cron_name),
        boundary: None,
        stop_on_fail: false,
        actions: vec![action],
        rules: None,
        cw20_coins: vec![],
    };

    let msg = WasmMsg::Execute {
        contract_addr: croncat_addr.to_string(),
        msg: to_binary(&task_request)?,
        funds: vec![],
    };
    Ok(Response::new()
        .add_attribute("action", "create_versioner_cron_task")
        .add_message(msg))
}
