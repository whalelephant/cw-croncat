#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::to_binary;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
#[cfg(not(feature = "library"))]
use cw2::set_contract_version;
use mod_sdk::helpers::query_wasm_smart_raw;
use mod_sdk::types::QueryResponse;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::types::dao::{ProposalListResponse, ProposalResponse, QueryDao, Status};
use crate::types::CheckProposalStatus;

// version info for migration info
const CONTRACT_NAME: &str = "croncat:mod-dao";
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
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Err(ContractError::Noop)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CheckProposalStatus(CheckProposalStatus {
            dao_address,
            proposal_id,
            status,
        }) => to_binary(&query_dao_proposal_status(
            deps,
            dao_address,
            proposal_id,
            status,
        )?),
        QueryMsg::CheckPassedProposals { dao_address } => {
            to_binary(&query_dao_proposals(deps, dao_address)?)
        }
    }
}

/// Query: CheckProposalStatus
/// Used as a helper method to check the proposals status
///
/// Response: QueryResponse
/// Returns true if the proposal status matches with the given `status`
/// Data is the information about the proposal
fn query_dao_proposal_status(
    deps: Deps,
    dao_address: String,
    proposal_id: u64,
    status: Status,
) -> StdResult<QueryResponse> {
    let dao_addr = deps.api.addr_validate(&dao_address)?;
    let bin = query_wasm_smart_raw(
        deps,
        dao_addr,
        to_binary(&QueryDao::Proposal { proposal_id })?,
    )?;

    let resp: ProposalResponse = cosmwasm_std::from_binary(&bin)?;
    Ok(QueryResponse {
        result: resp.proposal.status == status,
        data: bin,
    })
}

/// Query: CheckPassedProposals
/// Used as a helper method to check if there're any passed proposals
///
/// Response: QueryResponse
/// Returns true if there's at least one passed proposal
/// Data contains a vector of passed proposals
fn query_dao_proposals(deps: Deps, dao_address: String) -> StdResult<QueryResponse> {
    let dao_addr = deps.api.addr_validate(&dao_address)?;
    // Query the amount of proposals
    let proposal_count = deps
        .querier
        .query_wasm_smart(dao_addr.clone(), &QueryDao::ProposalCount {})?;
    let res: ProposalListResponse = deps.querier.query_wasm_smart(
        dao_addr,
        &QueryDao::ListProposals {
            start_after: None,
            limit: Some(proposal_count),
        },
    )?;

    for proposal_response in &res.proposals {
        if proposal_response.proposal.status == Status::Passed {
            return Ok(QueryResponse {
                result: true,
                data: to_binary(&proposal_response.id)?,
            });
        }
    }
    Ok(QueryResponse {
        result: false,
        data: to_binary(&res.proposals)?,
    })
}
