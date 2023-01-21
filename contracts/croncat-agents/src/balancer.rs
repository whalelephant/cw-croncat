use std::os::macos::raw::stat;

use cosmwasm_std::{Addr, Deps, Env, Storage, Uint128, Uint64};
use croncat_sdk_agents::msg::AgentTaskResponse;
use croncat_sdk_tasks::types::SlotType;

use crate::{
    error::ContractError,
    state::{AGENTS_ACTIVE, AGENT_STATS},
};

pub trait Balancer<'a> {
    #[doc = r".Gets agent tasks count for block/cron slots
    # Errors
    This function will return an error if agent does not exists"]
    fn get_agent_tasks(
        &self,
        deps: &Deps,
        env: &Env,
        agent_id: Addr,
        slot_items: (Option<u64>, Option<u64>),
    ) -> Result<Option<AgentTaskResponse>, ContractError>;
    #[doc = r"Removes balancer stats and rebalances"]
    fn on_agent_unregistered(
        &self,
        storage: &'a mut dyn Storage,
        agent_id: Addr,
    ) -> Result<(), ContractError>;

    #[doc = r"Updates agent stats when agent completed task on specified slot"]
    fn on_task_completed(
        &self,
        storage: &'a mut dyn Storage,
        _env: &Env,
        agent_id: &Addr,
        slot_type: SlotType,
    ) -> Result<(), ContractError>;
}

pub struct RoundRobinBalancer {}

impl RoundRobinBalancer {
    pub const fn new() -> RoundRobinBalancer {
        RoundRobinBalancer {}
    }
}
impl<'a> Balancer<'a> for RoundRobinBalancer {
    fn get_agent_tasks(
        &self,
        deps: &Deps,
        _env: &Env,
        agent_id: Addr,
        slot_items: (Option<u64>, Option<u64>),
    ) -> Result<Option<AgentTaskResponse>, ContractError> {
        let mut active = AGENTS_ACTIVE.load(deps.storage)?;
        if !active.contains(&agent_id) {
            return Err(ContractError::AgentNotRegistered {});
        }
        if slot_items == (None, None) {
            return Ok(None);
        }
        let agent_count = active.len() as u64;
        let (block_slots, cron_slots) = slot_items;

        let mut equalizer = |slot_type: SlotType,
                             total_tasks: u64|
         -> Result<(Uint64, Uint64), ContractError> {
            if total_tasks < 1 {
                return Ok((Uint64::zero(), Uint64::zero()));
            }
            //This sort is unstable (i.e., may reorder equal elements), in-place (i.e., does not allocate), and O(n log n) worst-case.
            //It is typically faster than stable sorting, except in a few special cases,
            //e.g., when the slice consists of several concatenated sorted sequences.
            active.sort_unstable_by(|left, right| {
                let stats1 = AGENT_STATS.load(deps.storage, left).unwrap_or_default();
                let stats2 = AGENT_STATS.load(deps.storage, right).unwrap_or_default();
                match slot_type {
                    SlotType::Block => stats1
                        .completed_block_tasks
                        .partial_cmp(&stats2.completed_block_tasks)
                        .unwrap(),
                    SlotType::Cron => stats1
                        .completed_cron_tasks
                        .partial_cmp(&stats2.completed_cron_tasks)
                        .unwrap(),
                }
            });
            let agent_diff_index = active
                .iter()
                .position(|x| x == &agent_id)
                .ok_or(ContractError::AgentNotRegistered {})?
                as u64;

            if total_tasks <= active.len() as u64 {
                let agent_tasks_total = 1u64
                    .saturating_sub(agent_diff_index.saturating_sub(total_tasks.saturating_sub(1)));
                Ok((agent_tasks_total.into(), agent_tasks_total.into()))
            } else {
                let leftover = total_tasks % agent_count;
                let mut extra = 0u64;
                if leftover > 0 {
                    extra = 1u64.saturating_sub(
                        agent_diff_index.saturating_sub(leftover.saturating_sub(1)),
                    );
                }
                let agent_tasks_total = total_tasks.saturating_div(agent_count) + extra;

                Ok((agent_tasks_total.into(), extra.into()))
            }
        };

        let (n, _) = equalizer(SlotType::Block, block_slots.unwrap_or_default())?;
        let num_block_tasks = n;

        let (n, _) = equalizer(SlotType::Cron, cron_slots.unwrap_or_default())?;
        let num_cron_tasks = n;

        Ok(Some(AgentTaskResponse {
            num_block_tasks,
            num_cron_tasks,
        }))
    }

    fn on_agent_unregistered(
        &self,
        storage: &'a mut dyn Storage,
        agent_id: Addr,
    ) -> Result<(), ContractError> {
        let active = AGENTS_ACTIVE.load(storage)?;
        if active.contains(&agent_id) {
            return Err(ContractError::AgentNotRegistered);
        }
        AGENT_STATS.remove(storage, &agent_id);
        Ok(())
    }

    fn on_task_completed(
        &self,
        storage: &'a mut dyn Storage,
        _env: &Env,
        agent_id: &Addr,
        slot_type: SlotType,
    ) -> Result<(), ContractError> {
        let mut stats = AGENT_STATS.may_load(storage, agent_id)?.unwrap_or_default();
        match slot_type {
            SlotType::Block => stats.completed_block_tasks += 1,
            SlotType::Cron => stats.completed_cron_tasks += 1,
        }
        AGENT_STATS.save(storage, agent_id, &stats)?;
        Ok(())
    }
}

impl Default for RoundRobinBalancer {
    fn default() -> RoundRobinBalancer {
        RoundRobinBalancer::new()
    }
}
