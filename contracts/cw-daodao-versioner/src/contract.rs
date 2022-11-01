use serde_cw_value::Value;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult
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
            dao_address,
            chain_id,
        } => query_registrations(deps, dao_address, chain_id),
        QueryMsg::GetRegistration {
            name: _,
            chain_id: _,
            version: _,
        } => todo!(),
        QueryMsg::GetCodeIdInfo {
            chain_id: _,
            code_id: _,
        } => todo!(),
    }
}

pub fn query_result(_deps: DepsMut, _info: MessageInfo) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "query_result"))
}

fn query_registrations(deps: Deps, dao_address: String, chain_id: String) -> StdResult<Binary> {
    let dao_addr = deps.api.addr_validate(&dao_address)?;
    let res: Binary = deps.querier.query_wasm_smart(
        dao_addr,
        &QueryMsg::ListRegistrations {
            dao_address,
            chain_id,
        },
    )?;
    Ok(res)
}
fn create_version_check_task(){

}
