use crate::state::Config;
use crate::state::TaskIndexes;
use crate::{slots, ContractError::AgentNotRegistered};
use cosmwasm_std::Uint64;
use cosmwasm_std::{Addr, Deps, Env, StdError, StdResult, Storage};
use cw_croncat_core::msg::AgentTaskResponse;
use cw_croncat_core::types::{Agent, SlotType, Task};
use cw_storage_plus::IndexedMap;
use cw_storage_plus::Item;

#[derive(PartialEq)]
pub enum BalancerMode {
    ActivationOrder,
    Equalizer,
}
pub trait Balancer<'a> {
    fn get_agent_tasks(
        &mut self,
        deps: Deps,
        env: Env,
        config: &Item<'a, Config>,
        active_agents: &Item<'a, Vec<Addr>>,
        agent_id: Addr,
        slot_items: (Option<u64>, Option<u64>),
    ) -> StdResult<Option<AgentTaskResponse>>;
}

pub struct RoundRobinBalancer {
    pub mode: BalancerMode,
}

impl<'a> RoundRobinBalancer {
    pub fn default() -> RoundRobinBalancer {
        return RoundRobinBalancer::new(BalancerMode::ActivationOrder);
    }
    pub fn new(mode: BalancerMode) -> RoundRobinBalancer {
        return RoundRobinBalancer { mode };
    }
    fn update_or_append(
        &self,
        overflows: &mut Vec<(SlotType, u32, u32)>,
        value: (SlotType, u32, u32),
    ) {
        match overflows
            .iter_mut()
            .find(|ref p| p.0 == value.0 && p.1 == value.1)
        {
            Some(found) => {
                found.2 += value.2;
            }
            None => {
                overflows.push(value);
            }
        }
    }
}
impl<'a> Balancer<'a> for RoundRobinBalancer {
    fn get_agent_tasks(
        &mut self,
        deps: Deps,
        env: Env,
        config: &Item<'a, Config>,
        active_agents: &Item<'a, Vec<Addr>>,
        agent_id: Addr,
        slot_items: (Option<u64>, Option<u64>),
    ) -> StdResult<Option<AgentTaskResponse>> {
        let conf: Config = config.load(deps.storage)?;
        let active = active_agents.load(deps.storage)?;
        if !active.contains(&agent_id) {
            return Err(StdError::GenericErr {
                msg: AgentNotRegistered {}.to_string(),
            });
        }
        let agent_active_indices_config = conf.agent_active_indices;
        let agent_active_indices: Vec<usize> = (0..active.len()).collect();
        let agent_index = active
            .iter()
            .position(|x| x == &agent_id)
            .expect("Agent not active or not registered!");

        if slot_items == (None, None) {
            return Ok(None);
        }
        let mut num_block_tasks = Uint64::from(0u64);
        let mut num_cron_tasks = Uint64::from(0u64);

        match self.mode {
            BalancerMode::ActivationOrder => {
                if let Some(current_block_task_total) = slot_items.0 {
                    if current_block_task_total <= active.len() as u64 {
                        num_block_tasks = (current_block_task_total - agent_index as u64).into();
                    } else {
                        let leftover = current_block_task_total % active.len() as u64;
                    }
                }
                if let Some(current_cron_task_total) = slot_items.1 {
                    if current_cron_task_total <= active.len() as u64 {}
                }

                Ok(Some(AgentTaskResponse {
                    num_block_tasks,
                    num_cron_tasks,
                }))
            }
            BalancerMode::Equalizer => todo!(),
        }
    }
}
