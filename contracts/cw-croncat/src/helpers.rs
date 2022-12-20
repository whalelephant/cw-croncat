use crate::state::{Config, QueueItem};
// use cosmwasm_std::Binary;
// use cosmwasm_std::StdError;
// use thiserror::Error;

use crate::ContractError::AgentNotRegistered;
use crate::{ContractError, CwCroncat};
use cosmwasm_std::{
    coin, to_binary, Addr, Api, BankMsg, Coin, CosmosMsg, Env, StdResult, Storage, SubMsg, WasmMsg,
};
use cw20::{Cw20CoinVerified, Cw20ExecuteMsg};
use cw_croncat_core::msg::ExecuteMsg;
use cw_croncat_core::traits::{BalancesOperations, FindAndMutate};
use cw_croncat_core::types::{calculate_required_amount, AgentStatus};
pub use cw_croncat_core::types::{GenericBalance, Task};
//use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::ops::Div;
//use std::str::FromStr;
pub(crate) fn vect_difference<T: std::clone::Clone + std::cmp::PartialEq>(
    v1: &[T],
    v2: &[T],
) -> Vec<T> {
    v1.iter().filter(|&x| !v2.contains(x)).cloned().collect()
}

// pub(crate) fn from_raw_str(value: &str) -> Option<Coin> {
//     let re = Regex::new(r"^([0-9.]+)([a-z][a-z0-9]*)$").unwrap();
//     assert!(re.is_match(value));
//     let caps = re.captures(value)?;
//     let amount = caps.get(1).map_or("", |m| m.as_str());
//     let denom = caps.get(2).map_or("", |m| m.as_str());
//     if denom.len() < 3 || denom.len() > 128{
//         return Option::None;
//     }
//     Some(Coin::new(u128::from_str(amount).unwrap(), denom))
// }

// Helper to distribute funds/tokens
pub(crate) fn send_tokens(
    to: &Addr,
    balance: &GenericBalance,
) -> StdResult<(Vec<SubMsg>, GenericBalance)> {
    let native_balance = &balance.native;
    let mut coins: GenericBalance = GenericBalance::default();
    let mut msgs: Vec<SubMsg> = if native_balance.is_empty() {
        vec![]
    } else {
        coins.native = native_balance.to_vec();
        vec![SubMsg::new(BankMsg::Send {
            to_address: to.into(),
            amount: native_balance.to_vec(),
        })]
    };

    let cw20_balance = &balance.cw20;
    let cw20_msgs: StdResult<Vec<_>> = cw20_balance
        .iter()
        .map(|c| {
            let msg = Cw20ExecuteMsg::Transfer {
                recipient: to.into(),
                amount: c.amount,
            };
            let exec = SubMsg::new(WasmMsg::Execute {
                contract_addr: c.address.to_string(),
                msg: to_binary(&msg)?,
                funds: vec![],
            });
            Ok(exec)
        })
        .collect();
    coins.cw20 = cw20_balance.to_vec();
    msgs.append(&mut cw20_msgs?);
    Ok((msgs, coins))
}

/// has_cw_coins returns true if the list of CW20 coins has at least the required amount
pub(crate) fn has_cw_coins(coins: &[Cw20CoinVerified], required: &Cw20CoinVerified) -> bool {
    coins
        .iter()
        .find(|c| c.address == required.address)
        .map(|m| m.amount >= required.amount)
        .unwrap_or(false)
}

impl<'a> CwCroncat<'a> {
    pub fn get_agent_status(
        &self,
        storage: &dyn Storage,
        env: Env,
        account_id: Addr,
    ) -> Result<AgentStatus, ContractError> {
        let c: Config = self.config.load(storage)?;
        let active = self.agent_active_queue.load(storage)?;

        // Pending
        let mut pending_iter = self.agent_pending_queue.iter(storage)?;
        // If agent is pending, Check if they should get nominated to checkin to become active
        let agent_position = if let Some(pos) = pending_iter.position(|address| {
            if let Ok(addr) = address {
                addr == account_id
            } else {
                false
            }
        }) {
            pos
        } else {
            // Check for active
            if active.contains(&account_id) {
                return Ok(AgentStatus::Active);
            } else {
                return Err(AgentNotRegistered {});
            }
        };

        // Edge case if last agent unregistered
        if active.is_empty() && agent_position == 0 {
            return Ok(AgentStatus::Nominated);
        };

        // Load config's task ratio, total tasks, active agents, and agent_nomination_begin_time.
        // Then determine if this agent is considered "Nominated" and should call CheckInAgent
        let max_agent_index =
            self.max_agent_nomination_index(storage, &c, env, &(active.len() as u64))?;
        let agent_status = match max_agent_index {
            Some(max_idx) if agent_position as u64 <= max_idx => AgentStatus::Nominated,
            _ => AgentStatus::Pending,
        };
        Ok(agent_status)
    }

    /// Calculate the biggest index of nomination for pending agents
    pub(crate) fn max_agent_nomination_index(
        &self,
        storage: &dyn Storage,
        cfg: &Config,
        env: Env,
        num_active_agents: &u64,
    ) -> Result<Option<u64>, ContractError> {
        let block_time = env.block.time.seconds();

        let agent_nomination_begin_time = self.agent_nomination_begin_time.load(storage)?;

        match agent_nomination_begin_time {
            Some(begin_time) => {
                let min_tasks_per_agent = cfg.min_tasks_per_agent;
                let total_tasks = self.task_total(storage)?;
                let num_agents_to_accept =
                    self.agents_to_let_in(&min_tasks_per_agent, num_active_agents, &total_tasks);

                if num_agents_to_accept > 0 {
                    let time_difference = block_time - begin_time.seconds();

                    let max_index = cmp::max(
                        time_difference.div(cfg.agent_nomination_duration as u64),
                        num_agents_to_accept - 1,
                    );
                    Ok(Some(max_index))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    pub fn agents_to_let_in(
        &self,
        max_tasks: &u64,
        num_active_agents: &u64,
        total_tasks: &u64,
    ) -> u64 {
        let num_tasks_covered = num_active_agents * max_tasks;
        if total_tasks > &num_tasks_covered {
            // It's possible there are more "covered tasks" than total tasks,
            // so use saturating subtraction to hit zero and not go below
            let total_tasks_needing_agents = total_tasks.saturating_sub(num_tasks_covered);
            let remainder = if total_tasks_needing_agents % max_tasks == 0 {
                0
            } else {
                1
            };
            total_tasks_needing_agents / max_tasks + remainder
        } else {
            0
        }
    }

    // Change balances of task and contract if action did transaction that went through
    pub fn task_after_action(
        &self,
        storage: &mut dyn Storage,
        api: &dyn Api,
        queue_item: QueueItem,
        ok: bool,
    ) -> Result<Task, ContractError> {
        let task_hash = queue_item.task_hash.unwrap();
        let mut task = self.get_task_by_hash(storage, &task_hash)?;
        if ok {
            let mut config = self.config.load(storage)?;
            let action_idx = queue_item.action_idx;
            let action = &task.actions[action_idx as usize];

            // update task balances and contract balances
            if let Some(sent) = action.bank_sent() {
                task.total_deposit.native.checked_sub_coins(sent)?;
                config.available_balance.checked_sub_native(sent)?;
            } else if let Some(sent) = action.cw20_sent(api) {
                task.total_deposit.cw20.find_checked_sub(&sent)?;
                config.available_balance.cw20.find_checked_sub(&sent)?;
            };
            self.config.save(storage, &config)?;
            if task.with_queries() {
                self.tasks_with_queries.save(storage, &task_hash, &task)?;
            } else {
                self.tasks.save(storage, &task_hash, &task)?;
            }
        }
        Ok(task)
    }
}

/// Generate submsgs for this proxy call and the price for it
pub(crate) fn proxy_call_submsgs_price(
    task: &Task,
    cfg: Config,
    next_idx: u64,
) -> Result<(Vec<SubMsg>, Coin), ContractError> {
    let (sub_msgs, gas_total) = task.get_submsgs_with_total_gas(
        cfg.gas_base_fee,
        cfg.gas_action_fee,
        cfg.gas_query_fee,
        cfg.gas_wasm_query_fee,
        next_idx,
    )?;
    let gas_amount = calculate_required_amount(gas_total, cfg.agent_fee)?;
    let price_amount = cfg.gas_fraction.calculate(gas_amount, 1)?;
    let price = coin(price_amount, cfg.native_denom);
    Ok((sub_msgs, price))
}
/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }

    // /// Get Count
    // pub fn count<Q, T, CQ>(&self, querier: &Q) -> StdResult<CountResponse>
    // where
    //     Q: Querier,
    //     T: Into<String>,
    //     CQ: CustomQuery,
    // {
    //     let msg = QueryMsg::GetCount {};
    //     let query = WasmQuery::Smart {
    //         contract_addr: self.addr().into(),
    //         msg: to_binary(&msg)?,
    //     }
    //     .into();
    //     let res: CountResponse = QuerierWrapper::<CQ>::new(querier).query(&query)?;
    //     Ok(res)
    // }
}
