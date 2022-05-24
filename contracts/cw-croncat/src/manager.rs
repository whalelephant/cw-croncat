use crate::error::ContractError;
use crate::state::{Config, CwCroncat, QueueItem};
use cosmwasm_std::{
    Addr, DepsMut, Empty, Env, MessageInfo, Reply, Response, StdResult, Storage, SubMsg,
};
use cw_croncat_core::types::{Agent, SlotType};

impl<'a> CwCroncat<'a> {
    // TODO:
    /// Executes a task based on the current task slot
    /// Computes whether a task should continue further or not
    /// Makes a cross-contract call with the task configuration
    /// Called directly by a registered agent
    pub fn proxy_call(
        &mut self,
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
    ) -> Result<Response, ContractError> {
        let c: Config = self.config.load(deps.storage)?;
        if c.paused {
            return Err(ContractError::CustomError {
                val: "Contract paused".to_string(),
            });
        }

        // only registered agent signed, because micropayments will benefit long term
        let agent_opt = self.agents.may_load(deps.storage, info.sender.clone())?;
        if agent_opt.is_none() {
            return Err(ContractError::AgentNotRegistered {});
        }
        let active_agents: Vec<Addr> = self
            .agent_active_queue
            .may_load(deps.storage)?
            .unwrap_or_default();

        // make sure agent is active
        if !active_agents.contains(&info.sender) {
            return Err(ContractError::AgentNotRegistered {});
        }
        let agent = agent_opt.unwrap();

        // get slot items, find the next task hash available
        // if empty slot found, let agent get paid for helping keep house clean
        let slot = self.get_current_slot_items(&env.block, deps.storage);
        let (slot_id, slot_kind) = slot.unwrap();
        let some_hash = self.pop_slot_item(deps.storage, &slot_id, &slot_kind);
        if some_hash.is_none() {
            self.send_base_agent_reward(deps.storage, agent);
            return Err(ContractError::NoTaskFound {});
        }

        // Get the task details
        // if no task, exit and reward agent.
        let hash = some_hash.unwrap();
        let some_task = self.tasks.may_load(deps.storage, hash.clone())?;
        if some_task.is_none() {
            self.send_base_agent_reward(deps.storage, agent);
            return Err(ContractError::NoTaskFound {});
        }

        // ----------------------------------------------------
        // TODO: FINISH!!!!!!
        // AGENT Task Allowance Logic: see line 339
        // ----------------------------------------------------

        let task = some_task.unwrap();

        // TODO: Bring this back!
        // // Fee breakdown:
        // // - Used Gas: Task Txn Fee Cost
        // // - Agent Fee: Incentivize Execution SLA
        // //
        // // Task Fee Examples:
        // // Total Fee = Gas Fee + Agent Fee
        // // Total Balance = Task Deposit + Total Fee
        // //
        // // NOTE: Gas cost includes the cross-contract call & internal logic of this contract.
        // // Direct contract gas fee will be lower than task execution costs, however
        // // we require the task owner to appropriately estimate gas for overpayment.
        // // The gas overpayment will also accrue to the agent since there is no way to read
        // // how much gas was actually used on callback.
        // let call_fee_used = u128::from(task.gas).saturating_mul(self.gas_price);
        // let call_total_fee = call_fee_used.saturating_add(self.agent_fee);
        // let call_total_balance = task.deposit.0.saturating_add(call_total_fee);

        // // safety check and not burn too much gas.
        // if call_total_balance > task.total_deposit.0 {
        //     log!("Not enough task balance to execute task, exiting");
        //     // Process task exit, if no future task can execute
        //     return self.exit_task(hash);
        // }

        // TODO: Bring this back!
        // // Update agent storage
        // // Increment agent reward & task count
        // // Reward for agent MUST include the amount of gas used as a reimbursement
        // agent.balance = U128::from(agent.balance.0.saturating_add(call_total_fee));
        // agent.total_tasks_executed = U128::from(agent.total_tasks_executed.0.saturating_add(1));
        // self.available_balance = self.available_balance.saturating_sub(call_total_fee);

        // TODO: Bring this back!
        // // Reset missed slot, if any
        // if agent.last_missed_slot != 0 {
        //     agent.last_missed_slot = 0;
        // }
        // self.agents.insert(&env::signer_account_id(), &agent);

        // TODO: Bring this back!
        // // Decrease task balance, Update task storage
        // task.total_deposit = U128::from(task.total_deposit.0.saturating_sub(call_total_balance));
        // self.tasks.insert(&hash, &task);

        // TODO: Move to external rule query handler
        // Proceed to query loops if rules are found in the task
        // Each rule is chained into the next, then evaluated if response is true before proceeding
        // let mut rule_responses: Vec<Attribute> = vec![];
        // if task.rules.is_some() {
        //     let mut rule_success: bool = false;
        //     // let mut previous_msg: Option<Binary>;
        //     for (idx, rule) in task.clone().rules.unwrap().iter().enumerate() {
        //         let rule_res: RuleResponse<Option<Binary>> = deps
        //             .querier
        //             .query_wasm_smart(&rule.contract_addr, &rule.msg)?;
        //         println!("{:?}", rule_res);
        //         rule_success = rule_res.0;

        //         // TODO: needs better approach
        //         d.push(Attribute::new(idx.to_string(), format!("{:?}", rule_res.1)));
        //     }
        //     if !rule_success {
        //         return Err(ContractError::CustomError {
        //             val: "Rule evaluated to false".to_string(),
        //         });
        //     }
        // }

        // Setup submessages for actions for this task
        // Each submessage in storage, computes & stores the "next" reply to allow for chained message processing.
        let mut sub_msgs: Vec<SubMsg<Empty>> = vec![];
        let next_idx = self.rq_next_id(deps.storage)?;
        let actions = task.clone().actions;
        let self_addr = env.contract.address;

        // Add submessages for all actions
        for action in actions {
            let sub_msg: SubMsg = SubMsg::reply_always(action.msg, next_idx)
                .with_gas_limit(action.gas_limit.unwrap());

            sub_msgs.push(sub_msg);
        }

        // Keep track for later scheduling
        self.rq_push(
            deps.storage,
            QueueItem {
                prev_idx: None,
                task_hash: Some(hash),
                contract_addr: Some(self_addr),
            },
        )?;

        // Add the messages, reply handler responsible for task rescheduling
        let final_res = Response::new()
            .add_attribute("method", "proxy_call")
            .add_attribute("agent", info.sender)
            .add_attribute("slot_id", slot_id.to_string())
            .add_attribute("slot_kind", slot_id.to_string())
            .add_attribute("task_hash", task.to_hash())
            // .add_attributes(rule_responses)
            .add_submessages(sub_msgs);

        Ok(final_res)
    }

    /// Logic executed on the completion of a proxy call
    /// Reschedule next task
    pub(crate) fn proxy_callback(
        &self,
        deps: DepsMut,
        env: Env,
        _msg: Reply,
        task_hash: Vec<u8>,
    ) -> Result<Response, ContractError> {
        let mut response = Response::new().add_attribute("method", "proxy_callback");

        // reschedule next!
        if let Some(task) = self.tasks.may_load(deps.storage, task_hash)? {
            // TODO: How can we compute gas & fees paid on this txn?
            // let out_of_funds = call_total_balance > task.total_deposit;
            //  || out_of_funds
            // if out of balance or non-recurring, exit
            if !task.stop_on_fail {
                // Process task exit, if no future task can execute
                return self.remove_task(deps, task.to_hash());
            }

            // Parse interval into a future timestamp, then convert to a slot
            let (next_id, slot_kind) = task.interval.next(env, task.boundary);

            // If the next interval comes back 0, then this task should not schedule again
            if next_id == 0 {
                self.remove_task(deps, task.to_hash())?;
                return Err(ContractError::CustomError {
                    val: "Task ended".to_string(),
                });
            }

            response = response.add_attribute("next_slot", next_id.to_string());

            // Get previous task hashes in slot, add as needed
            let update_vec_data = |d: Option<Vec<Vec<u8>>>| -> StdResult<Vec<Vec<u8>>> {
                match d {
                    // has some data, simply push new hash
                    Some(data) => {
                        let mut s = data;
                        s.push(task.to_hash_vec());
                        Ok(s)
                    }
                    // No data, push new vec & hash
                    None => Ok(vec![task.to_hash_vec()]),
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
        } else {
            return Err(ContractError::NoTaskFound {});
        }

        Ok(response)
    }

    /// Internal management of agent reward
    /// Used in cases where there are empty slots or failed txns
    /// Keep the agent profitable, as this will be a business expense
    pub(crate) fn send_base_agent_reward(&self, _storage: &dyn Storage, _agent: Agent) {
        // let mut a = agent;
        // // reward agent for diligence
        // let agent_base_fee = self.agent_fee;
        // agent.balance = U128::from(agent.balance.0.saturating_add(agent_base_fee));
        // agent.total_tasks_executed = U128::from(agent.total_tasks_executed.0.saturating_add(1));
        // self.available_balance = self.available_balance.saturating_sub(agent_base_fee);

        // // Reset missed slot, if any
        // if agent.last_missed_slot != 0 {
        //     agent.last_missed_slot = 0;
        // }
        // self.agents.save(&env::signer_account_id(), &agent);
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
