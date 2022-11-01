#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary
};
use cw2::set_contract_version;


use crate::error::ContractError;
use crate::msg::*;
use crate::msg::dao_registry::{Query::*};

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
        // Echo
        ExecuteMsg::QueryResult {} => query_result(deps, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ListRegistrations {
            registrar_addr,
            chain_id,
        } =>to_binary(&query_registrations(deps, registrar_addr, chain_id)?),
        QueryMsg::GetRegistration {
            name: _,
            chain_id: _,
            version: _,
        } => todo!(),
        QueryMsg::GetCodeIdInfo {
            registrar_addr,
            chain_id,
            code_id,
        } => to_binary(&query_code_id_info(deps,registrar_addr, chain_id,code_id)?),
    }
}

pub fn query_result(_deps: DepsMut, _info: MessageInfo) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "query_result"))
}

fn query_registrations(deps: Deps, registrar_addr: String, chain_id: String) -> StdResult<ListRegistrationsResponse> {
    let registrar_address = deps.api.addr_validate(&registrar_addr)?;
    let res: ListRegistrationsResponse = deps.querier.query_wasm_smart(
        registrar_address,
        &QueryMsg::ListRegistrations {
            registrar_addr,
            chain_id,
        },
    )?;
    Ok(res)
}
fn query_code_id_info(deps: Deps, registrar_addr: String, chain_id: String,code_id: u64) -> StdResult<GetRegistrationResponse> {
    let registrar_address = deps.api.addr_validate(&registrar_addr)?;
    let res: GetRegistrationResponse = deps.querier.query_wasm_smart(
        registrar_address,
        &QueryMsg::GetCodeIdInfo {
            registrar_addr,
            chain_id,
            code_id
        },
    )?;
    Ok(res)
}
fn create_version_check_task(deps: Deps, dao_address: String, chain_id: String)->Result<Response, ContractError> {
    let regs=query_registrations(deps, dao_address, chain_id)?;
    todo!();
}
