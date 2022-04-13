use crate::error::ContractError;
use crate::slots::{Boundary, Interval};
use crate::state::{Config, CONFIG, TASKS};
use cosmwasm_std::{
    Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};
use cw20::Balance;
use hex::encode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Rule {
    /// TBD: Interchain query support (See ibc::IbcMsg)
    pub chain_id: Option<String>,

    /// Account to direct all view calls against
    pub contract_id: Addr,

    // NOTE: Only allow static pre-defined query msg
    pub msg: Binary,
}

/// The response required by all rule queries. Bool is needed for croncat, T allows flexible rule engine
pub type RuleResponse<T> = (bool, T);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Task {
    /// Entity responsible for this task, can change task details
    pub owner_id: Addr,

    /// Scheduling definitions
    pub interval: Interval,
    pub boundary: Boundary,

    /// Defines if this task can continue until balance runs out
    pub stop_on_fail: bool,

    /// NOTE: Only tally native balance here, manager can maintain token/balances outside of tasks
    pub total_deposit: Balance,

    /// The cosmos message to call, if time or rules are met
    pub action: CosmosMsg,
    // TODO: Decide if batch should be supported? Does that break gas limits ESP when rules are applied?
    // pub action: Vec<CosmosMsg>,
    /// A prioritized list of messages that can be chained decision matrix
    /// required to complete before task action
    /// Rules MUST return the ResolverResponse type
    pub rules: Option<Vec<Rule>>,
}

impl Task {
    /// Get the hash of a task based on parameters
    pub fn to_hash(&self) -> String {
        let message = format!(
            "{:?}{:?}{:?}{:?}{:?}",
            self.owner_id, self.interval, self.boundary, self.action, self.rules
        );

        let hash = Sha256::digest(message.as_bytes());
        encode(hash)
    }
    /// Check if a cadence string is valid by attempting to parse it
    pub fn valid_interval(&self) -> bool {
        // TODO: Use the schedule standard to validte

        true
    }
    // /// Returns the base amount required to execute 1 task
    // /// NOTE: this is not the final used amount, just the user-specified amount total needed
    // pub fn task_balance_uses(&self, task: &Task) -> u128 {
    //     task.deposit.0 + (u128::from(task.gas) * self.gas_price) + self.agent_fee
    // }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaskRequest {
    pub interval: Interval,
    pub boundary: Boundary,
    pub stop_on_fail: bool,
    pub action: CosmosMsg,
    pub rules: Option<Vec<Rule>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaskResponse {
    pub task_hash: String,
    pub owner_id: Addr,
    pub interval: Interval,
    pub boundary: Boundary,
    pub stop_on_fail: bool,
    pub total_deposit: Balance,
    pub action: CosmosMsg,
    pub rules: Option<Vec<Rule>>,
}

// TODO:
// /// Returns task data
// /// Used by the frontend for viewing tasks
// pub(crate) fn query_get_tasks(
//     _deps: Deps,
//     _slot: Option<u128>,
//     _from_index: Option<u64>,
//     _limit: Option<u64>,
// ) -> StdResult<Vec<TaskResponse>> {
//     // let active = AGENTS_ACTIVE_QUEUE.load(deps.storage)?;

//     Ok(vec![TaskResponse {}])
// }

// TODO:
// /// Returns task data for a specific owner
// pub(crate) fn query_get_tasks_by_owner(
//     _deps: Deps,
//     _owner_id: Addr,
// ) -> StdResult<Vec<TaskResponse>> {
//     // let active = AGENTS_ACTIVE_QUEUE.load(deps.storage)?;

//     Ok(vec![TaskResponse {}])
// }

// TODO:
/// Returns single task data
pub(crate) fn query_get_task(
    deps: Deps,
    task_hash: String,
    owner_id: Addr,
) -> StdResult<Option<TaskResponse>> {
    let res = TASKS.may_load(deps.storage, (task_hash.as_bytes().to_vec(), owner_id))?;
    if res.is_none() {
        return Ok(None);
    }

    let task: Task = res.unwrap();

    Ok(Some(TaskResponse {
        task_hash: task.to_hash(),
        owner_id: task.owner_id,
        interval: task.interval,
        boundary: task.boundary,
        stop_on_fail: task.stop_on_fail,
        total_deposit: task.total_deposit,
        action: task.action,
        rules: task.rules,
    }))
}

// TODO:
// /// Returns a hash computed by the input task data
// pub(crate) fn query_get_task_hash(
//     _deps: Deps,
//     _task: Task,
// ) -> StdResult<String> {
//     // let active = AGENTS_ACTIVE_QUEUE.load(deps.storage)?;

//     Ok("")
// }

// TODO:
// /// Returns task data
// /// Used by the frontend for viewing tasks
// pub(crate) fn query_validate_interval(
//     _deps: Deps,
//     _interval: Interval,
// ) -> StdResult<bool> {
//     // let active = AGENTS_ACTIVE_QUEUE.load(deps.storage)?;

//     Ok(false)
// }

/// Allows any user or contract to pay for future txns based on a specific schedule
/// contract, function id & other settings. When the task runs out of balance
/// the task is no longer executed, any additional funds will be returned to task owner.
pub fn create_task(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    task: TaskRequest,
) -> Result<Response, ContractError> {
    if info.funds.is_empty() {
        return Err(ContractError::CustomError {
            val: "Must attach funds".to_string(),
        });
    }
    let c: Config = CONFIG.load(deps.storage)?;
    if c.paused {
        return Err(ContractError::CustomError {
            val: "Create task paused".to_string(),
        });
    }

    // TODO: What other msg types are needed to validate against
    // Additional checks - needs to protect against scripting owner / self situations
    match task.action.clone() {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            funds: _,
            msg: _,
        }) => {
            if contract_addr == env.contract.address {
                // TODO: How to guard this??
                // check that the method is NOT the callback of this contract
                // assert!(
                //     function_id != "proxy_callback",
                //     "Function id invalid"
                // );
                // cannot be THIS contract id, unless predecessor is owner of THIS contract
                if info.sender == c.owner_id {
                    return Err(ContractError::CustomError {
                        val: "Creator invalid".to_string(),
                    });
                }
            }
        }
        _ => unreachable!(),
    }

    let item = Task {
        owner_id: info.sender,
        interval: task.interval,
        boundary: task.boundary,
        stop_on_fail: task.stop_on_fail,
        total_deposit: Balance::from(info.funds),
        action: task.action,
        rules: task.rules,
    };

    if !item.valid_interval() {
        return Err(ContractError::CustomError {
            val: "Interval invalid".to_string(),
        });
    }

    // // Check that balance is sufficient for 1 execution minimum
    // let call_balance_used = self.task_balance_uses(&item);
    // let min_balance_needed: u128 = if recurring == Some(true) {
    //     call_balance_used * 2
    // } else {
    //     call_balance_used
    // };
    // assert!(
    //     min_balance_needed <= item.total_deposit.0,
    //     "Not enough task balance to execute job, need at least {}",
    //     min_balance_needed
    // );

    let hash = item.to_hash();

    // // Parse cadence into a future timestamp, then convert to a slot
    // let schedule = Schedule::new(&item);
    // let slot = Slot::new(&item);
    // TODO: Check if next_id is 0, if so, panic
    // let (next_id, slot_kind) = item.interval.next(env, item.boundary);

    // // Add task to catalog
    // assert!(
    //     self.tasks.insert(&hash.0, &item).is_none(),
    //     "Task already exists"
    // );

    // // Get previous task hashes in slot, add as needed
    // let mut slot_slots = self.slots.get(&next_slot).unwrap_or(Vec::new());
    // slot_slots.push(hash.0.clone());
    // log!("Task next slot: {}", next_slot);
    // self.slots.insert(&next_slot, &slot_slots);

    // // Keep track of which tasks are owned by whom
    // let mut owner_tasks = self.task_owners.get(&item.owner_id).unwrap_or(Vec::new());
    // owner_tasks.push(hash.0.clone());
    // log!("Task owner list: {}", item.owner_id);
    // self.task_owners.insert(&item.owner_id, &owner_tasks);

    // // Add the attached balance into available_balance
    // self.available_balance = self
    //     .available_balance
    //     .saturating_add(env::attached_deposit());

    // TODO:
    Ok(Response::new()
        .add_attribute("method", "create_task")
        .add_attribute("task_hash", hash))
}

// TODO:
/// Deletes a task in its entirety, returning any remaining balance to task owner.
pub fn remove_task(
    _deps: DepsMut,
    _info: MessageInfo,
    _env: Env,
    _task_hash: String,
) -> Result<Response, ContractError> {
    // TODO:
    Ok(Response::new().add_attribute("method", "remove_task"))
}

// TODO:
/// Refill a task with more balance to continue its execution
/// NOTE: Sending balance here for a task that doesnt exist will result in loss of funds, or you could just use this as an opportunity for donations :D
/// NOTE: Currently restricting this to owner only, so owner can make sure the task ends
pub fn refill_task(
    _deps: DepsMut,
    _info: MessageInfo,
    _env: Env,
    _task_hash: String,
) -> Result<Response, ContractError> {
    // TODO:
    Ok(Response::new().add_attribute("method", "refill_task"))
}

// TODO:
/// Executes a task based on the current task slot
/// Computes whether a task should continue further or not
/// Makes a cross-contract call with the task configuration
/// Called directly by a registered agent
pub fn proxy_call(
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
    _deps: DepsMut,
    _info: MessageInfo,
    _env: Env,
    _task_hash: String,
    _current_slot: u64,
) -> Result<Response, ContractError> {
    // TODO:
    Ok(Response::new().add_attribute("method", "proxy_callback"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{coins, BankMsg, CosmosMsg};
    use cw20::Balance;

    #[test]
    fn task_to_hash_success() {
        let to_address = String::from("you");
        let amount = coins(1015, "earth");
        let bank = BankMsg::Send { to_address, amount };
        let msg: CosmosMsg = bank.clone().into();

        let task = Task {
            owner_id: Addr::unchecked("nobody".to_string()),
            interval: Interval::Immediate,
            boundary: Boundary {
                start: None,
                end: None,
            },
            stop_on_fail: false,
            total_deposit: Balance::default(),
            action: msg,
            rules: None,
        };

        // HASH IT!
        let hash = task.to_hash();
        assert_eq!(
            "2e87eb9d9dd92e5a903eacb23ce270676e80727bea1a38b40646be08026d05bc",
            hash
        );
    }
}
