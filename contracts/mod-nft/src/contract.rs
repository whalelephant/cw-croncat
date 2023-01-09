#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
#[cfg(not(feature = "library"))]
use cw2::set_contract_version;
use cw721::Cw721QueryMsg::{OwnerOf, Tokens};
use cw721::{OwnerOfResponse, TokensResponse};
use mod_sdk::types::QueryResponse;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::types::OwnerOfNft;
use crate::ContractError;

// version info for migration info
const CONTRACT_NAME: &str = "croncat:mod-nft";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, StdError> {
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
        QueryMsg::OwnerOfNft(OwnerOfNft {
            address,
            nft_address,
            token_id,
        }) => to_binary(&query_nft_owner(deps, address, nft_address, token_id)?),
        QueryMsg::AddrHasNft {
            address,
            nft_address,
        } => to_binary(&query_addr_has_nft(deps, address, nft_address)?),
    }
}

/// Query: OwnerOfNft
/// Used as a helper method to check if address is the owner
/// of the token with specific id in the nft_address contract
///
/// Response: QueryResponse
/// Returns true if address owns the token
/// Data contains information about the owner and approvals found for this token
/// Return error if token_id or nft_address are wrong
fn query_nft_owner(
    deps: Deps,
    address: String,
    nft_address: String,
    token_id: String,
) -> StdResult<QueryResponse> {
    let valid_nft = deps.api.addr_validate(&nft_address)?;
    let res: OwnerOfResponse = deps.querier.query_wasm_smart(
        valid_nft,
        &OwnerOf {
            token_id,
            include_expired: None,
        },
    )?;
    Ok(QueryResponse {
        result: address == res.owner,
        data: to_binary(&res.owner)?,
    })
}

/// Query: OwnerOfNft
/// Used as a helper method to check if address owns any nft tokens
///
/// Response: QueryResponse
/// Returns true if address owns at least one token for this contracts
/// Data is empty
/// Return error if nft_address is wrong
fn query_addr_has_nft(
    deps: Deps,
    address: String,
    nft_address: String,
) -> StdResult<QueryResponse> {
    let valid_nft = deps.api.addr_validate(&nft_address)?;
    let res: TokensResponse = deps.querier.query_wasm_smart(
        valid_nft,
        &Tokens {
            owner: address,
            start_after: None,
            limit: None,
        },
    )?;
    Ok(QueryResponse {
        result: !res.tokens.is_empty(),
        data: to_binary(&res.tokens)?,
    })
}
