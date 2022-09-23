use cosmwasm_std::{
    coins,
    testing::{mock_env, mock_info},
    DepsMut, Empty, Response,
};
use cw_croncat_core::msg::InstantiateMsg;

use crate::{ContractError, CwCroncat};

pub fn mock_init(store: &CwCroncat, deps: DepsMut<Empty>) -> Result<Response, ContractError> {
    let msg = InstantiateMsg {
        denom: "atom".to_string(),
        owner_id: None,
        gas_base_fee: None,
        agent_nomination_duration: Some(360),
        cw_rules_addr: "todo".to_string(),
    };
    let info = mock_info("creator", &coins(1000, "meow"));
    store.instantiate(deps, mock_env(), info.clone(), msg)
}
