use crate::error::ContractError;
use crate::slots::{Boundary, Interval, SlotType};
use crate::state::{Config, CwCroncat};
use cosmwasm_std::{
    Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, GovMsg, IbcMsg, MessageInfo, Order,
    Response, StdResult, WasmMsg,
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
            self.owner_id,
            self.interval,
            self.clone().boundary,
            self.action,
            self.rules
        );

        let hash = Sha256::digest(message.as_bytes());
        encode(hash)
    }
    /// Get the hash of a task based on parameters
    pub fn to_hash_vec(&self) -> Vec<u8> {
        self.to_hash().into_bytes()
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

impl<'a> CwCroncat<'a> {
    /// Returns task data
    /// Used by the frontend for viewing tasks
    pub(crate) fn query_get_tasks(
        &self,
        deps: Deps,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> StdResult<Vec<TaskResponse>> {
        let mut ret: Vec<TaskResponse> = Vec::new();
        let mut start = 0;
        let size: u64 = self
            .task_total
            .may_load(deps.storage)?
            .unwrap_or(100)
            .min(1000);
        if let Some(index) = from_index {
            start = index;
        }
        let mut end = u64::min(start.saturating_add(100), size);
        if let Some(l) = limit {
            end = u64::min(start.saturating_add(l), size);
        }

        // NOTE: could setup another index to allow efficient paginated
        let keys: Vec<Vec<u8>> = self
            .tasks
            .keys(deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;

        for i in start..end {
            let task_hash = &keys[i as usize];
            let res = self.tasks.may_load(deps.storage, task_hash.to_vec())?;
            if let Some(task) = res {
                ret.push(TaskResponse {
                    task_hash: task.to_hash(),
                    owner_id: task.owner_id,
                    interval: task.interval,
                    boundary: task.boundary,
                    stop_on_fail: task.stop_on_fail,
                    total_deposit: task.total_deposit,
                    action: task.action,
                    rules: task.rules,
                });
            }
        }

        Ok(ret)
    }

    /// Returns task data for a specific owner
    pub(crate) fn query_get_tasks_by_owner(
        &self,
        deps: Deps,
        owner_id: Addr,
    ) -> StdResult<Vec<TaskResponse>> {
        let tasks_by_owner: Vec<TaskResponse> = self
            .tasks
            .idx
            .owner
            .prefix(owner_id)
            .range(deps.storage, None, None, Order::Ascending)
            .map(|x| {
                x.map(|(_, task)| TaskResponse {
                    task_hash: task.to_hash(),
                    owner_id: task.owner_id,
                    interval: task.interval,
                    boundary: task.boundary,
                    stop_on_fail: task.stop_on_fail,
                    total_deposit: task.total_deposit,
                    action: task.action,
                    rules: task.rules,
                })
            })
            .collect::<StdResult<Vec<_>>>()?;

        Ok(tasks_by_owner)
    }

    /// Returns single task data
    pub(crate) fn query_get_task(
        &self,
        deps: Deps,
        task_hash: String,
    ) -> StdResult<Option<TaskResponse>> {
        let res = self
            .tasks
            .may_load(deps.storage, task_hash.as_bytes().to_vec())?;
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

    /// Returns a hash computed by the input task data
    pub(crate) fn query_get_task_hash(&self, task: Task) -> StdResult<String> {
        Ok(task.to_hash())
    }

    /// Check if interval params are valid by attempting to parse
    pub(crate) fn query_validate_interval(&self, interval: Interval) -> StdResult<bool> {
        Ok(interval.is_valid())
    }

    /// Gets a set of tasks.
    /// Default: Returns the next executable set of tasks hashes.
    ///
    /// Optional Parameters:
    /// "offset" - An unsigned integer specifying how far in the future to check for tasks that are slotted.
    ///
    /// Result:
    /// (block id, block task hash's, time id, time task hash's)
    pub(crate) fn query_slot_tasks(
        &self,
        deps: Deps,
        slot: Option<u64>,
    ) -> StdResult<(u64, Vec<String>, u64, Vec<String>)> {
        let mut block_id: u64 = 0;
        let mut block_hashes: Vec<Vec<u8>> = Vec::new();
        let mut time_id: u64 = 0;
        let mut time_hashes: Vec<Vec<u8>> = Vec::new();

        // Check if slot was supplied, otherwise get the next slots for block and time
        if let Some(id) = slot {
            block_hashes = self
                .block_slots
                .may_load(deps.storage, id)?
                .unwrap_or_default();
            if !block_hashes.is_empty() {
                block_id = id;
            }
            time_hashes = self
                .block_slots
                .may_load(deps.storage, id)?
                .unwrap_or_default();
            if !time_hashes.is_empty() {
                time_id = id;
            }
        } else {
            let time: Vec<(u64, _)> = self
                .time_slots
                .range(deps.storage, None, None, Order::Ascending)
                .take(1)
                .collect::<StdResult<Vec<(u64, _)>>>()?;

            if !time.is_empty() {
                // (time_id, time_hashes) = time[0].clone();
                let slot = time[0].clone();
                time_id = slot.0;
                time_hashes = slot.1;
            }

            let block: Vec<(u64, _)> = self
                .block_slots
                .range(deps.storage, None, None, Order::Ascending)
                .take(1)
                .collect::<StdResult<Vec<(u64, _)>>>()?;

            if !block.is_empty() {
                // (block_id, block_hashes) = block[0].clone();
                let slot = block[0].clone();
                block_id = slot.0;
                block_hashes = slot.1;
            }
        }

        // Generate strings for all hashes
        let b_hashes: Vec<_> = block_hashes
            .iter()
            .map(|b| String::from_utf8(b.to_vec()).unwrap_or_else(|_| "".to_string()))
            .collect();
        let t_hashes: Vec<_> = time_hashes
            .iter()
            .map(|t| String::from_utf8(t.to_vec()).unwrap_or_else(|_| "".to_string()))
            .collect();

        Ok((block_id, b_hashes, time_id, t_hashes))
    }

    /// Gets list of active slot ids, for both time & block slots
    /// (time, block)
    pub(crate) fn query_slot_ids(&self, deps: Deps) -> StdResult<(Vec<u64>, Vec<u64>)> {
        let time: Vec<u64> = self
            .time_slots
            .keys(deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;
        let block: Vec<u64> = self
            .block_slots
            .keys(deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;
        Ok((time, block))
    }

    /// Allows any user or contract to pay for future txns based on a specific schedule
    /// contract, function id & other settings. When the task runs out of balance
    /// the task is no longer executed, any additional funds will be returned to task owner.
    pub fn create_task(
        &self,
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
        let c: Config = self.config.load(deps.storage)?;
        if c.paused {
            return Err(ContractError::CustomError {
                val: "Create task paused".to_string(),
            });
        }

        // TODO: Finish checking other msg types needing validation
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
                    // TODO: Is there any way sender can be "self" creating a malicious task?
                    if info.sender != c.owner_id {
                        return Err(ContractError::CustomError {
                            val: "Creator invalid".to_string(),
                        });
                    }
                }
            }
            CosmosMsg::Bank(BankMsg::Send { .. }) => {
                // Restrict bank msg for time being, so contract doesnt get drained, however could allow an escrow type setup
                return Err(ContractError::CustomError {
                    val: "Bank send disabled".to_string(),
                });
            }
            CosmosMsg::Bank(BankMsg::Burn { .. }) => {
                // Restrict bank msg for time being, so contract doesnt get drained, however could allow an escrow type setup
                return Err(ContractError::CustomError {
                    val: "Bank burn disabled".to_string(),
                });
            }
            CosmosMsg::Gov(GovMsg::Vote { .. }) => {
                // Restrict bank msg for time being, so contract doesnt get drained, however could allow an escrow type setup
                return Err(ContractError::CustomError {
                    val: "Gov module disabled".to_string(),
                });
            }
            CosmosMsg::Ibc(IbcMsg::Transfer { .. }) => {
                // Restrict bank msg for time being, so contract doesnt get drained, however could allow an escrow type setup
                return Err(ContractError::CustomError {
                    val: "Ibc transfer disabled".to_string(),
                });
            }
            // TODO: Check authZ messages
            _ => (),
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

        if !item.interval.is_valid() {
            return Err(ContractError::CustomError {
                val: "Interval invalid".to_string(),
            });
        }

        // TODO:
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

        // Parse interval into a future timestamp, then convert to a slot
        let (next_id, slot_kind) = item.interval.next(env, item.boundary);

        // If the next interval comes back 0, then this task should not schedule again
        if next_id == 0 {
            return Err(ContractError::CustomError {
                val: "Task ended".to_string(),
            });
        }

        // Add task to catalog
        self.tasks
            .update(deps.storage, item.to_hash_vec(), |old| match old {
                Some(_) => Err(ContractError::CustomError {
                    val: "Task already exists".to_string(),
                }),
                None => Ok(item.clone()),
            })?;

        // Increment task totals
        let size: u64 = self.task_total.may_load(deps.storage)?.unwrap_or(0);
        self.task_total.save(deps.storage, &(size + 1))?;

        // Get previous task hashes in slot, add as needed
        let update_vec_data = |d: Option<Vec<Vec<u8>>>| -> StdResult<Vec<Vec<u8>>> {
            match d {
                // has some data, simply push new hash
                Some(data) => {
                    let mut s = data;
                    s.push(item.to_hash_vec());
                    Ok(s)
                }
                // No data, push new vec & hash
                None => Ok(vec![item.to_hash_vec()]),
            }
        };

        // Based on slot kind, put into block or cron slots
        match slot_kind {
            SlotType::Block => {
                self.block_slots
                    .update(deps.storage, next_id, update_vec_data)?;
            }
            SlotType::Cron => {
                self.time_slots
                    .update(deps.storage, next_id, update_vec_data)?;
            }
        }

        // TODO:
        // // Keep track of which tasks are owned by whom
        // let mut owner_tasks = self.task_owners.get(&item.owner_id).unwrap_or(Vec::new());
        // owner_tasks.push(hash.0.clone());
        // log!("Task owner list: {}", item.owner_id);
        // self.task_owners.insert(&item.owner_id, &owner_tasks);

        // TODO:
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
        &self,
        deps: DepsMut,
        _info: MessageInfo,
        _env: Env,
        task_hash: String,
    ) -> Result<Response, ContractError> {
        let hash_vec = task_hash.into_bytes();
        let task_raw = self.tasks.may_load(deps.storage, hash_vec.clone())?;
        if task_raw.is_none() {
            return Err(ContractError::CustomError {
                val: "No task found by hash".to_string(),
            });
        }
        // let task = task_raw.unwrap();
        // let owner_id = task.owner_id;

        // Remove all the thangs
        self.tasks.remove(deps.storage, hash_vec)?;

        // TODO:
        // find any scheduled things and remove them!
        // check which type of slot it would be in, then iterate to remove

        Ok(Response::new().add_attribute("method", "remove_task"))
    }

    // TODO: FINISH
    /// Refill a task with more balance to continue its execution
    /// NOTE: Sending balance here for a task that doesnt exist will result in loss of funds, or you could just use this as an opportunity for donations :D
    /// NOTE: Currently restricting this to owner only, so owner can make sure the task ends
    pub fn refill_task(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        _env: Env,
        task_hash: String,
    ) -> Result<Response, ContractError> {
        let hash_vec = task_hash.into_bytes();
        let task_raw = self.tasks.may_load(deps.storage, hash_vec)?;
        if task_raw.is_none() {
            return Err(ContractError::CustomError {
                val: "Task already exists".to_string(),
            });
        }
        let task = task_raw.unwrap();
        if task.owner_id != info.sender {
            return Err(ContractError::CustomError {
                val: "Only owner can refill their task".to_string(),
            });
        }

        // TODO:
        // // Add the attached balance into available_balance
        // self.available_balance = self
        //     .available_balance
        //     .saturating_add(env::attached_deposit());

        // TODO: report how full the task is total
        Ok(Response::new().add_attribute("method", "refill_task"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use cosmwasm_std::testing::MockStorage;
    use cosmwasm_std::{coin, coins, to_binary, Addr, BankMsg, CosmosMsg, Empty, StakingMsg};
    use cw20::Balance;
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
    // use crate::error::ContractError;
    use crate::helpers::CwTemplateContract;
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::slots::BoundarySpec;

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::entry::execute,
            crate::entry::instantiate,
            crate::entry::query,
        );
        Box::new(contract)
    }

    const ADMIN: &str = "ADMIN";
    const ANYONE: &str = "ANYONE";
    const NATIVE_DENOM: &str = "atom";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            let accounts: Vec<(u128, String)> =
                vec![(100, ADMIN.to_string()), (100, ANYONE.to_string())];
            for (amt, address) in accounts.iter() {
                router
                    .bank
                    .init_balance(
                        storage,
                        &Addr::unchecked(address),
                        vec![coin(amt.clone(), NATIVE_DENOM.to_string())],
                    )
                    .unwrap();
            }
        })
    }

    fn proper_instantiate() -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());
        let owner_addr = Addr::unchecked(ADMIN);

        let msg = InstantiateMsg {
            denom: "atom".to_string(),
            owner_id: Some(owner_addr.clone()),
        };
        let cw_template_contract_addr = app
            .instantiate_contract(cw_template_id, owner_addr, &msg, &[], "Manager", None)
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr);

        (app, cw_template_contract)
    }

    #[test]
    fn query_task_hash_success() {
        let (app, cw_template_contract) = proper_instantiate();
        let contract_addr = cw_template_contract.addr();

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

        // HASH CHECK!
        let task_hash: String = app
            .wrap()
            .query_wasm_smart(
                &contract_addr.clone(),
                &QueryMsg::GetTaskHash {
                    task: Box::new(task),
                },
            )
            .unwrap();
        assert_eq!(
            "2e87eb9d9dd92e5a903eacb23ce270676e80727bea1a38b40646be08026d05bc",
            task_hash
        );
    }

    #[test]
    fn query_validate_interval_success() {
        let (app, cw_template_contract) = proper_instantiate();
        let contract_addr = cw_template_contract.addr();

        let intervals: Vec<Interval> = vec![
            Interval::Once,
            Interval::Immediate,
            Interval::Block(12345),
            Interval::Cron("0 0 * * * *".to_string()),
        ];
        for i in intervals.iter() {
            let valid: bool = app
                .wrap()
                .query_wasm_smart(
                    &contract_addr.clone(),
                    &QueryMsg::ValidateInterval {
                        interval: i.to_owned(),
                    },
                )
                .unwrap();
            assert!(valid);
        }
    }

    #[test]
    fn query_get_tasks() {
        let (mut app, cw_template_contract) = proper_instantiate();
        let contract_addr = cw_template_contract.addr();

        let validator = String::from("you");
        let amount = coin(3, "atom");
        let stake = StakingMsg::Delegate { validator, amount };
        let msg: CosmosMsg = stake.clone().into();

        let create_task_msg = ExecuteMsg::CreateTask {
            task: TaskRequest {
                interval: Interval::Immediate,
                boundary: Boundary {
                    start: None,
                    end: None,
                },
                stop_on_fail: false,
                action: msg,
                rules: None,
            },
        };

        // create a task
        app.execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &create_task_msg,
            &coins(37, "atom"),
        )
        .unwrap();

        // check storage has the task
        let all_tasks: Vec<TaskResponse> = app
            .wrap()
            .query_wasm_smart(
                &contract_addr.clone(),
                &QueryMsg::GetTasks {
                    from_index: None,
                    limit: None,
                },
            )
            .unwrap();
        assert_eq!(all_tasks.len(), 1);

        let owner_tasks: Vec<TaskResponse> = app
            .wrap()
            .query_wasm_smart(
                &contract_addr.clone(),
                &QueryMsg::GetTasksByOwner {
                    owner_id: Addr::unchecked(ANYONE),
                },
            )
            .unwrap();
        assert_eq!(owner_tasks.len(), 1);
    }

    #[test]
    fn check_task_create_fail_cases() -> StdResult<()> {
        let (mut app, cw_template_contract) = proper_instantiate();
        let contract_addr = cw_template_contract.addr();

        let validator = String::from("you");
        let amount = coin(3, "atom");
        let stake = StakingMsg::Delegate { validator, amount };
        let msg: CosmosMsg = stake.clone().into();

        let create_task_msg = ExecuteMsg::CreateTask {
            task: TaskRequest {
                interval: Interval::Immediate,
                boundary: Boundary {
                    start: None,
                    end: None,
                },
                stop_on_fail: false,
                action: msg.clone(),
                rules: None,
            },
        };
        // let task_id_str = "be93bba6f619350950985f6e3498d1aa54e276b7db8f7c5bfbfe2998f5fbce3f".to_string();
        // let task_id = task_id_str.clone().into_bytes();

        // Must attach funds
        let res_err = app
            .execute_contract(
                Addr::unchecked(ANYONE),
                contract_addr.clone(),
                &create_task_msg,
                &vec![],
            )
            .unwrap_err();
        assert_eq!(
            ContractError::CustomError {
                val: "Must attach funds".to_string()
            },
            res_err.downcast().unwrap()
        );

        // Create task paused
        let change_settings_msg = ExecuteMsg::UpdateSettings {
            paused: Some(true),
            owner_id: None,
            // treasury_id: None,
            agent_fee: None,
            agent_task_ratio: None,
            agents_eject_threshold: None,
            gas_price: None,
            proxy_callback_gas: None,
            slot_granularity: None,
        };
        app.execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &change_settings_msg,
            &vec![],
        )
        .unwrap();
        let res_err = app
            .execute_contract(
                Addr::unchecked(ANYONE),
                contract_addr.clone(),
                &create_task_msg,
                &coins(13, "atom"),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::CustomError {
                val: "Create task paused".to_string()
            },
            res_err.downcast().unwrap()
        );
        // Set it back
        app.execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &ExecuteMsg::UpdateSettings {
                paused: Some(false),
                owner_id: None,
                // treasury_id: None,
                agent_fee: None,
                agent_task_ratio: None,
                agents_eject_threshold: None,
                gas_price: None,
                proxy_callback_gas: None,
                slot_granularity: None,
            },
            &vec![],
        )
        .unwrap();

        // Creator invalid
        let action_self = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.clone().into_string(),
            funds: vec![],
            msg: to_binary(&change_settings_msg.clone())?,
        });
        let res_err = app
            .execute_contract(
                Addr::unchecked(ANYONE),
                contract_addr.clone(),
                &ExecuteMsg::CreateTask {
                    task: TaskRequest {
                        interval: Interval::Once,
                        boundary: Boundary {
                            start: None,
                            end: None,
                        },
                        stop_on_fail: false,
                        action: action_self.clone(),
                        rules: None,
                    },
                },
                &coins(13, "atom"),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::CustomError {
                val: "Creator invalid".to_string()
            },
            res_err.downcast().unwrap()
        );

        // Interval invalid
        let res_err = app
            .execute_contract(
                Addr::unchecked(ANYONE),
                contract_addr.clone(),
                &ExecuteMsg::CreateTask {
                    task: TaskRequest {
                        interval: Interval::Cron("faux_paw".to_string()),
                        boundary: Boundary {
                            start: None,
                            end: None,
                        },
                        stop_on_fail: false,
                        action: msg.clone(),
                        rules: None,
                    },
                },
                &coins(13, "atom"),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::CustomError {
                val: "Interval invalid".to_string()
            },
            res_err.downcast().unwrap()
        );

        // Task already exists
        app.execute_contract(
            Addr::unchecked(ANYONE),
            contract_addr.clone(),
            &create_task_msg,
            &coins(13, "atom"),
        )
        .unwrap();
        let res_err = app
            .execute_contract(
                Addr::unchecked(ANYONE),
                contract_addr.clone(),
                &create_task_msg,
                &coins(13, "atom"),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::CustomError {
                val: "Task already exists".to_string()
            },
            res_err.downcast().unwrap()
        );

        // Task ended
        let res_err = app
            .execute_contract(
                Addr::unchecked(ANYONE),
                contract_addr.clone(),
                &ExecuteMsg::CreateTask {
                    task: TaskRequest {
                        interval: Interval::Block(12346),
                        boundary: Boundary {
                            start: None,
                            end: Some(BoundarySpec::Height(1)),
                        },
                        stop_on_fail: false,
                        action: msg.clone(),
                        rules: None,
                    },
                },
                &coins(13, "atom"),
            )
            .unwrap_err();
        assert_eq!(
            ContractError::CustomError {
                val: "Task ended".to_string()
            },
            res_err.downcast().unwrap()
        );

        // TODO: (needs impl!) Not enough task balance to execute job

        Ok(())
    }

    #[test]
    fn check_task_create_success() -> StdResult<()> {
        let (mut app, cw_template_contract) = proper_instantiate();
        let contract_addr = cw_template_contract.addr();

        let validator = String::from("you");
        let amount = coin(3, "atom");
        let stake = StakingMsg::Delegate { validator, amount };
        let msg: CosmosMsg = stake.clone().into();

        let create_task_msg = ExecuteMsg::CreateTask {
            task: TaskRequest {
                interval: Interval::Immediate,
                boundary: Boundary {
                    start: None,
                    end: None,
                },
                stop_on_fail: false,
                action: msg,
                rules: None,
            },
        };
        let task_id_str =
            "be93bba6f619350950985f6e3498d1aa54e276b7db8f7c5bfbfe2998f5fbce3f".to_string();

        // create a task
        let res = app
            .execute_contract(
                Addr::unchecked(ANYONE),
                contract_addr.clone(),
                &create_task_msg,
                &coins(37, "atom"),
            )
            .unwrap();
        // Assert task hash is returned as part of event attributes
        let mut has_created_hash: bool = false;
        for e in res.events {
            for a in e.attributes {
                if a.key == "task_hash" && a.value == task_id_str.clone() {
                    has_created_hash = true;
                }
            }
        }
        assert!(has_created_hash);

        // check storage has the task
        let new_task: Option<TaskResponse> = app
            .wrap()
            .query_wasm_smart(
                &contract_addr.clone(),
                &QueryMsg::GetTask {
                    task_hash: task_id_str.clone(),
                },
            )
            .unwrap();
        assert!(new_task.is_some());
        if let Some(t) = new_task {
            assert_eq!(Addr::unchecked(ANYONE), t.owner_id);
            assert_eq!(Interval::Immediate, t.interval);
            assert_eq!(
                Boundary {
                    start: None,
                    end: None,
                },
                t.boundary
            );
            assert_eq!(false, t.stop_on_fail);
            assert_eq!(Balance::from(coins(37, "atom")), t.total_deposit);
            assert_eq!(task_id_str.clone(), t.task_hash);
        }

        // get slot ids
        let slot_ids: (Vec<u64>, Vec<u64>) = app
            .wrap()
            .query_wasm_smart(&contract_addr.clone(), &QueryMsg::GetSlotIds {})
            .unwrap();
        let s_1: Vec<u64> = Vec::new();
        assert_eq!(s_1, slot_ids.0);
        assert_eq!(vec![12346], slot_ids.1);

        // get slot hashs
        let slot_info: (u64, Vec<String>, u64, Vec<String>) = app
            .wrap()
            .query_wasm_smart(
                &contract_addr.clone(),
                &QueryMsg::GetSlotHashes { slot: None },
            )
            .unwrap();
        let s_3: Vec<String> = Vec::new();
        assert_eq!(12346, slot_info.0);
        assert_eq!(vec![task_id_str.clone()], slot_info.1);
        assert_eq!(0, slot_info.2);
        assert_eq!(s_3, slot_info.3);

        Ok(())
    }
}
