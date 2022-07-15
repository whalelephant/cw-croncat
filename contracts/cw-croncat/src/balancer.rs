use crate::helpers::*;
use crate::state::Config;
use crate::{slots, ContractError::AgentNotRegistered};
use cosmwasm_std::Uint64;
use cosmwasm_std::{Addr, Deps, Env, StdError, StdResult, Storage};
use cw_croncat_core::msg::AgentTaskResponse;
use cw_croncat_core::types::{Agent, SlotType, Task};
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
        let agent_count = active.len() as u64;
        let agent_active_indices_config = conf.agent_active_indices;
        let agent_active_indices: Vec<usize> = (0..active.len()).collect();
        let agent_index = active
            .iter()
            .position(|x| x == &agent_id)
            .expect("Agent not active or not registered!") as u64;

        if slot_items == (None, None) {
            return Ok(None);
        }
        let mut num_block_tasks = Uint64::from(0u64);
        let mut num_cron_tasks = Uint64::from(0u64);

        match self.mode {
            BalancerMode::ActivationOrder => {
                let activation_ordering = |total_tasks: u64| -> Uint64 {
                    if total_tasks <= active.len() as u64 {
                        let agent_tasks_total = (1 as u64)
                            .saturating_sub(agent_index.saturating_sub(total_tasks.saturating_sub(1)));
                        agent_tasks_total.into()
                    } else {
                        let leftover = total_tasks % agent_count;
                        let mut rich_agents = agent_active_indices_config.clone();
                        rich_agents.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
                        let rich_indices: Vec<usize> =
                            rich_agents.iter().map(|v| v.1 as usize).collect();

                        let mut diff = vect_difference(&agent_active_indices, &rich_indices);
                        diff.extend(rich_indices);
                        let agent_index = diff
                            .iter()
                            .position(|x| x == &(agent_index as usize))
                            .expect("Agent not active or not registered!")
                            as u64;

                        let agent_tasks_total =total_tasks.saturating_div(agent_count)+ (1 as u64)
                            .saturating_sub(agent_index.saturating_sub(leftover.saturating_sub(1)));
                        agent_tasks_total.into()
                    }
                };

                if let Some(current_block_task_total) = slot_items.0 {
                    num_block_tasks = activation_ordering(current_block_task_total);
                }
                if let Some(current_cron_task_total) = slot_items.1 {
                    num_cron_tasks = activation_ordering(current_cron_task_total);
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
