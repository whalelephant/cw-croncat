use crate::state::{Config, TaskInfo};
use crate::{helpers::*, ContractError};
use cosmwasm_std::Uint64;
use cosmwasm_std::{Addr, Env};
use cosmwasm_std::{Deps, Storage};
use cw_croncat_core::msg::AgentTaskResponse;
use cw_croncat_core::types::{BalancerMode, SlotType};
use cw_storage_plus::Item;

pub trait Balancer<'a> {
    fn get_agent_tasks(
        &mut self,
        deps: &Deps,
        env: &Env,
        config: &Item<'a, Config>,
        active_agents: &Item<'a, Vec<Addr>>,
        agent_id: Addr,
        slot_items: (Option<u64>, Option<u64>),
    ) -> Result<Option<AgentTaskResponse>, ContractError>;
    fn on_agent_unregister(
        &self,
        storage: &'a mut dyn Storage,
        config: &Item<'a, Config>,
        active_agents: &Item<'a, Vec<Addr>>,
        agent_id: Addr,
    ) -> Result<(), ContractError>;
    fn on_task_completed(
        &self,
        storage: &'a mut dyn Storage,
        _env: &Env,
        config: &Item<'a, Config>,
        active_agents: &Item<'a, Vec<Addr>>,
        task_info: &TaskInfo,
    ) -> Result<(), ContractError>;
}

pub struct RoundRobinBalancer {
    pub mode: BalancerMode,
}

impl RoundRobinBalancer {
    pub fn new(mode: BalancerMode) -> RoundRobinBalancer {
        RoundRobinBalancer { mode }
    }
    pub(crate) fn update_or_append(
        &self,
        overflows: &mut Vec<(SlotType, u32, u32)>,
        value: (SlotType, u32, u32),
    ) {
        match overflows
            .iter_mut()
            .find(|p| p.0 == value.0 && p.1 == value.1)
        {
            Some(found) => {
                found.2 += value.2;
            }
            None => {
                overflows.push(value);
            }
        }
    }
    fn remove_agent_and_rebalance(
        &self,
        indices: &mut Vec<(SlotType, u32, u32)>,
        agent_index: u32,
    ) {
        indices.clear();
        let mut vec: Vec<(SlotType, u32, u32)> = Vec::new();
        for p in indices.iter() {
            match agent_index {
                aind if aind < p.1 => vec.push((p.0, p.1 - 1, p.2)),
                aind if aind > p.1 => vec.push((p.0, p.1, p.2)),
                _ => (),
            }
        }
        indices.extend(vec);
    }
}
impl<'a> Balancer<'a> for RoundRobinBalancer {
    fn get_agent_tasks(
        &mut self,
        deps: &Deps,
        _env: &Env,
        config: &Item<'a, Config>,
        active_agents: &Item<'a, Vec<Addr>>,
        agent_id: Addr,
        slot_items: (Option<u64>, Option<u64>),
    ) -> Result<Option<AgentTaskResponse>, ContractError> {
        let conf: Config = config.load(deps.storage)?;
        let active = active_agents.load(deps.storage)?;
        if !active.contains(&agent_id) {
            return Err(ContractError::AgentNotRegistered {});
        }
        let agent_count = active.len() as u64;
        let agent_active_indices_config = conf.agent_active_indices;
        let agent_active_indices: Vec<usize> = (0..active.len()).collect();
        let agent_index = active
            .iter()
            .position(|x| x == &agent_id)
            .ok_or(ContractError::AgentNotRegistered {})? as u64;

        if slot_items == (None, None) {
            return Ok(None);
        }
        let mut num_block_tasks = Uint64::zero();
        let mut num_block_tasks_extra = Uint64::zero();

        let mut num_cron_tasks = Uint64::zero();
        let mut num_cron_tasks_extra = Uint64::zero();

        match self.mode {
            BalancerMode::ActivationOrder => {
                let activation_ordering =
                    |total_tasks: u64| -> Result<(Uint64, Uint64), ContractError> {
                        if total_tasks < 1 {
                            return Ok((Uint64::zero(), Uint64::zero()));
                        }
                        if total_tasks <= active.len() as u64 {
                            let agent_tasks_total = 1u64.saturating_sub(
                                agent_index.saturating_sub(total_tasks.saturating_sub(1)),
                            );
                            Ok((agent_tasks_total.into(), Uint64::zero()))
                        } else {
                            let leftover = total_tasks % agent_count;

                            let mut rich_agents: Vec<(SlotType, u32, u32)> =
                                agent_active_indices_config
                                    .clone()
                                    .into_iter()
                                    .filter(|e| e.2 > 0)
                                    .collect::<_>();

                            rich_agents.sort_by(|a, b| a.2.cmp(&b.2));
                            let rich_indices: Vec<usize> =
                                rich_agents.iter().map(|v| v.1 as usize).collect();

                            let mut diff = vect_difference(&agent_active_indices, &rich_indices);
                            diff.extend(rich_indices);
                            let agent_index = diff
                                .iter()
                                .position(|x| x == &(agent_index as usize))
                                .ok_or(ContractError::AgentNotRegistered {})?
                                as u64;

                            let mut extra = 0u64;
                            if leftover > 0 {
                                extra = 1u64.saturating_sub(
                                    agent_index.saturating_sub(leftover.saturating_sub(1)),
                                );
                            }
                            let agent_tasks_total = total_tasks.saturating_div(agent_count) + extra;

                            Ok((agent_tasks_total.into(), extra.into()))
                        }
                    };

                if let Some(current_block_task_total) = slot_items.0 {
                    let (n, ne) = activation_ordering(current_block_task_total)?;
                    num_block_tasks = n;
                    num_block_tasks_extra = ne;
                }
                if let Some(current_cron_task_total) = slot_items.1 {
                    let (n, ne) = activation_ordering(current_cron_task_total)?;
                    num_cron_tasks = n;
                    num_cron_tasks_extra = ne;
                }

                Ok(Some(AgentTaskResponse {
                    num_block_tasks,
                    num_block_tasks_extra,
                    num_cron_tasks,
                    num_cron_tasks_extra,
                }))
            }
            BalancerMode::Equalizer => {
                let equalizer = |total_tasks: u64| -> Result<(Uint64, Uint64), ContractError> {
                    if total_tasks < 1 {
                        return Ok((Uint64::zero(), Uint64::zero()));
                    }
                    let mut rich_agents: Vec<(SlotType, u32, u32)> = agent_active_indices_config
                        .clone()
                        .into_iter()
                        .filter(|e| e.2 > 0)
                        .collect();

                    rich_agents.sort_by(|a, b| a.2.cmp(&b.2));
                    let rich_indices: Vec<usize> =
                        rich_agents.iter().map(|v| v.1 as usize).collect();

                    let mut diff = vect_difference(&agent_active_indices, &rich_indices);
                    diff.extend(rich_indices);

                    let agent_index = diff
                        .iter()
                        .position(|x| x == &(agent_index as usize))
                        .ok_or(ContractError::AgentNotRegistered {})?
                        as u64;

                    if total_tasks <= diff.len() as u64 {
                        let agent_tasks_total = 1u64.saturating_sub(
                            agent_index.saturating_sub(total_tasks.saturating_sub(1)),
                        );
                        Ok((agent_tasks_total.into(), agent_tasks_total.into()))
                    } else {
                        let leftover = total_tasks % agent_count;
                        let mut extra = 0u64;
                        if leftover > 0 {
                            extra = 1u64.saturating_sub(
                                agent_index.saturating_sub(leftover.saturating_sub(1)),
                            );
                        }
                        let agent_tasks_total = total_tasks.saturating_div(agent_count) + extra;

                        Ok((agent_tasks_total.into(), extra.into()))
                    }
                };

                if let Some(current_block_task_total) = slot_items.0 {
                    let (n, ne) = equalizer(current_block_task_total)?;
                    num_block_tasks = n;
                    num_block_tasks_extra = ne;
                }
                if let Some(current_cron_task_total) = slot_items.1 {
                    let (n, ne) = equalizer(current_cron_task_total)?;
                    num_cron_tasks = n;
                    num_cron_tasks_extra = ne;
                }

                Ok(Some(AgentTaskResponse {
                    num_block_tasks,
                    num_block_tasks_extra,
                    num_cron_tasks,
                    num_cron_tasks_extra,
                }))
            }
        }
    }

    fn on_agent_unregister(
        &self,
        storage: &'a mut dyn Storage,
        config: &Item<'a, Config>,
        active_agents: &Item<'a, Vec<Addr>>,
        agent_id: Addr,
    ) -> Result<(), ContractError> {
        let mut conf: Config = config.load(storage)?;
        let indices = conf.agent_active_indices.as_mut();
        let active = active_agents.load(storage)?;
        let agent_index = active
            .iter()
            .position(|x| x == &agent_id)
            .ok_or(ContractError::AgentNotRegistered {})? as u32;

        self.remove_agent_and_rebalance(indices, agent_index);

        config.save(storage, &conf)?;
        Ok(())
    }

    fn on_task_completed(
        &self,
        storage: &'a mut dyn Storage,
        _env: &Env,
        config: &Item<'a, Config>,
        active_agents: &Item<'a, Vec<Addr>>,
        task_info: &TaskInfo,
    ) -> Result<(), ContractError> {
        if !task_info.task_is_extra.unwrap_or(false) && self.mode == BalancerMode::ActivationOrder {
            return Ok(());
        };

        let mut conf: Config = config.load(storage)?;
        let indices = conf.agent_active_indices.as_mut();
        let active = active_agents.load(storage)?;
        let agent_id = &task_info.agent_id;
        let slot_kind = task_info.slot_kind;
        let agent_index = active
            .iter()
            .position(|x| x == agent_id)
            .ok_or(ContractError::AgentNotRegistered {})? as u32;

        self.update_or_append(indices, (slot_kind, agent_index, 1));
        config.save(storage, &conf)?;
        Ok(())
    }
}

impl Default for RoundRobinBalancer {
    fn default() -> RoundRobinBalancer {
        RoundRobinBalancer::new(BalancerMode::ActivationOrder)
    }
}
