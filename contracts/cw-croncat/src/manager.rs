use crate::error::ContractError;
use crate::state::CwCroncat;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

impl<'a> CwCroncat<'a> {
    // TODO:
    /// Executes a task based on the current task slot
    /// Computes whether a task should continue further or not
    /// Makes a cross-contract call with the task configuration
    /// Called directly by a registered agent
    pub fn proxy_call(
        &self,
        _deps: DepsMut,
        _info: MessageInfo,
        _env: Env,
    ) -> Result<Response, ContractError> {
        // TODO:
        Ok(Response::new().add_attribute("method", "proxy_call"))
    }

    // TODO:
    /// Logic executed on the completion of a proxy call
    /// Reschedule next task
    pub fn proxy_callback(
        &self,
        _deps: DepsMut,
        _info: MessageInfo,
        _env: Env,
        _task_hash: String,
        _current_slot: u64,
    ) -> Result<Response, ContractError> {
        // TODO:
        Ok(Response::new().add_attribute("method", "proxy_callback"))
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::{coins, BankMsg, CosmosMsg};
//     use cw20::Balance;

//     #[test]
//     fn task_to_hash_success() {
//         let to_address = String::from("you");
//         let amount = coins(1015, "earth");
//         let bank = BankMsg::Send { to_address, amount };
//         let msg: CosmosMsg = bank.clone().into();

//         let task = Task {
//             owner_id: Addr::unchecked("nobody".to_string()),
//             interval: Interval::Immediate,
//             boundary: Boundary {
//                 start: None,
//                 end: None,
//             },
//             stop_on_fail: false,
//             total_deposit: Balance::default(),
//             action: msg,
//             rules: None,
//         };

//         // HASH IT!
//         let hash = task.to_hash();
//         assert_eq!(
//             "2e87eb9d9dd92e5a903eacb23ce270676e80727bea1a38b40646be08026d05bc",
//             hash
//         );
//     }
// }
