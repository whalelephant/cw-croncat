#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::dao_registry::Query::*;
use crate::msg::*;
use crate::state::{REGISTRAR_ADDR, VERSION_MAP};

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
    REGISTRAR_ADDR.save(deps.storage, &registrar_addr)?;

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
        ExecuteMsg::CreateContractVersioner { name, chain_id } => create_contract_versioner(deps,name,chain_id),
        ExecuteMsg::RemoveContractVersioner { name, chain_id } => remove_contract_versioner(deps,name,chain_id),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VerifyNewVersionAvailable { name, chain_id } => todo!(),
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
fn query_code_id_info(
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
fn query_registration(
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
        .may_load(deps.storage, (&name, &chain_id.clone()))?
        .is_some()
    {
        return Err(ContractError::ContractAlreadyRegistered(
            name.clone(),
            chain_id.clone(),
        ));
    }
    let registrar_addr = REGISTRAR_ADDR.load(deps.storage)?;
    let regs = query_registrations(deps.as_ref(), registrar_addr.to_string(), name.clone(), chain_id.clone())?;
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
        .may_load(deps.storage, (&name, &chain_id.clone()))?
        .is_none()
    {
        return Err(ContractError::ContractNotRegistered(
            name.clone(),
            chain_id.clone(),
        ));
    }
    let registrar_addr = REGISTRAR_ADDR.load(deps.storage)?;
    
    VERSION_MAP.remove(
        deps.storage,
        (&name, &chain_id)
    );

    Ok(Response::new()
        .add_attribute("action", "remove_contract_versioner")
        .add_attribute("contract_name", name)
        .add_attribute("chain_id", chain_id))
}

