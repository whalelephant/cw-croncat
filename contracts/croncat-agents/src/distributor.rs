use std::cmp::min;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Env, Order, Storage};
use croncat_sdk_agents::types::*;
use croncat_sdk_tasks::types::SlotType;

use crate::{
    error::*,
    state::{agent_map, AGENT_NOMINATION_CHECKPOINT},
};

pub(crate) struct AgentDistributor {}

#[cw_serde]
pub(crate) struct NominationCheckPoint {
    // Starting block height of nomination checkpoint
    start_block: Option<u64>,
    // Number of tasks created from last nomination
    tasks_advancement: u64,
}

impl AgentDistributor {
    pub(crate) const fn new() -> AgentDistributor {
        AgentDistributor {}
    }
    fn agents_pending(&self, storage: &dyn Storage) -> Result<Vec<(Addr, Agent)>, ContractError> {
        let pending: Vec<_> = agent_map()
            .idx
            .by_status
            .prefix(AgentStatus::Pending.to_string())
            .range(storage, None, None, Order::Ascending)
            .map(|x| x.map(|r| r.1))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(pending)
    }
    pub(crate) fn agents_active(
        &self,
        storage: &dyn Storage,
    ) -> Result<Vec<(Addr, Agent)>, ContractError> {
        let active: Vec<_> = agent_map()
            .idx
            .by_status
            .prefix(AgentStatus::Active.to_string())
            .range(storage, None, None, Order::Ascending)
            .map(|x| x.map(|r| r.1))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(active)
    }
    pub(crate) fn has_pending(&self, storage: &dyn Storage) -> Result<bool, ContractError> {
        Ok(agent_map()
            .idx
            .by_status
            .prefix(AgentStatus::Pending.to_string())
            .range(storage, None, None, Order::Ascending)
            .any(|t| t.is_ok()))
    }
    pub(crate) fn has_active(&self, storage: &dyn Storage) -> Result<bool, ContractError> {
        Ok(agent_map()
            .idx
            .by_status
            .prefix(AgentStatus::Active.to_string())
            .range(storage, None, None, Order::Ascending)
            .any(|t| t.is_ok()))
    }

    // Registers new agent
    pub(crate) fn add_new_agent(
        &self,
        storage: &mut dyn Storage,
        env: &Env,
        agent_id: Addr,
        payable_account_id: Addr,
    ) -> Result<(Addr, Agent), ContractError> {
        let agent_status = if !self.has_active(storage)? {
            AgentStatus::Active //No active agents for now, make agent active
        } else {
            AgentStatus::Pending
        };

        let agent = Agent {
            payable_account_id,
            register_start: env.block.time,
            status: agent_status,
            completed_block_tasks: u64::default(),
            completed_cron_tasks: u64::default(),
            last_executed_slot: env.block.height,
        };
        self.add_agent_internal(storage, agent_id, agent)
    }
    pub(crate) fn get_agent(
        &self,
        storage: &dyn Storage,
        addr: &Addr,
    ) -> Result<Option<Agent>, ContractError> {
        let res = agent_map().may_load(storage, addr.as_bytes())?;
        Ok(res.map(|x| x.1))
    }

    pub(crate) fn get_agent_ids(
        &self,
        storage: &dyn Storage,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Result<(Vec<Addr>, Vec<Addr>), ContractError> {
        let active: Vec<_> = agent_map()
            .idx
            .by_status
            .prefix(AgentStatus::Active.to_string())
            .range_raw(storage, None, None, Order::Ascending)
            .skip(from_index.unwrap_or_default() as usize)
            .take(limit.unwrap_or(u64::MAX) as usize)
            .map(|x| x.map(|r| r.1 .0))
            .collect::<Result<Vec<_>, _>>()?;

        let pending: Vec<_> = agent_map()
            .idx
            .by_status
            .prefix(AgentStatus::Pending.to_string())
            .range_raw(storage, None, None, Order::Ascending)
            .skip(from_index.unwrap_or_default() as usize)
            .take(limit.unwrap_or(u64::MAX) as usize)
            .map(|x| x.map(|r| r.1 .0))
            .collect::<Result<Vec<_>, _>>()?;

        Ok((active, pending))
    }
    fn add_agent_internal(
        &self,
        storage: &mut dyn Storage,
        agent_id: Addr,
        agent: Agent,
    ) -> Result<(Addr, Agent), ContractError> {
        let result = agent_map().update(storage, agent_id.clone().as_bytes(), |old| match old {
            Some(_) => Err(ContractError::AgentAlreadyRegistered {}),
            None => Ok((agent_id, agent)),
        })?;
        Ok(result)
    }
    pub(crate) fn set_payable_account_id(
        &self,
        storage: &mut dyn Storage,
        agent_id: Addr,
        payable_account_id: Addr,
    ) -> Result<(), ContractError> {
        agent_map().update(storage, agent_id.clone().as_bytes(), |old| match old {
            Some(value) => {
                let mut agent = value.1;
                agent.payable_account_id = payable_account_id;
                Ok((agent_id, agent))
            }
            None => Err(ContractError::AgentNotRegistered {}),
        })?;
        Ok(())
    }
    fn set_agent_status(
        &self,
        storage: &mut dyn Storage,
        agent_id: Addr,
        status: AgentStatus,
    ) -> Result<(), ContractError> {
        agent_map().update(storage, agent_id.clone().as_bytes(), |old| match old {
            Some(value) => {
                let mut agent = value.1;
                agent.status = status;
                Ok((agent_id, agent))
            }
            None => Err(ContractError::AgentNotRegistered {}),
        })?;
        Ok(())
    }
    pub(crate) fn try_nominate_agent(
        &self,
        storage: &mut dyn Storage,
        env: &Env,
        config: &Config,
        agent_id: Addr,
    ) -> Result<(), ContractError> {
        // Agent must be in the pending queue
        // Get the position in the pending queue
        let checkpoint = AGENT_NOMINATION_CHECKPOINT.load(storage)?;
        let pending = self.agents_pending(storage)?;
        let pending_position = pending.iter().position(|a| a.0 == agent_id).ok_or(
            ContractError::AgentIsNotInPendingStatus {
                addr: agent_id.clone(),
            },
        )?;

        // edge case if last agent left
        if pending_position == 0 && !self.has_active(storage)? {
            self.set_agent_status(storage, agent_id, AgentStatus::Active)?;
            self.reset_nomination_checkpoint(storage)?;
            return Ok(());
        }
        // It works out such that the time difference between when this is called,
        // and the agent nomination begin time can be divided by the nomination
        // duration and we get an integer. We use that integer to determine if an
        // agent is allowed to get let in. If their position in the pending queue is
        // less than or equal to that integer, they get let in.
        let max_index = self
            .max_nomination_index(config, env, &checkpoint)?
            .ok_or(ContractError::TryLaterForNomination)?;

        if pending_position as u64 <= max_index {
            // Update state removing from pending queue
            for i in 0..pending_position {
                let rem = pending.get(i).unwrap();
                self.remove_agent(storage, &rem.0)?;
            }

            // Make this agent active
            self.set_agent_status(storage, agent_id, AgentStatus::Active)?;
            // and update the config, setting the nomination begin time to None,
            // which indicates no one will be nominated until more tasks arrive
            self.reset_nomination_checkpoint(storage)?;
        } else {
            return Err(ContractError::TryLaterForNomination);
        };
        Ok(())
    }

    pub(crate) fn remove_agent(
        &self,
        storage: &mut dyn Storage,
        agent_id: &Addr,
    ) -> Result<(), ContractError> {
        agent_map().remove(storage, agent_id.as_bytes())?;
        Ok(())
    }

    pub(crate) fn reset_nomination_checkpoint(
        &self,
        storage: &mut dyn Storage,
    ) -> Result<(), ContractError> {
        AGENT_NOMINATION_CHECKPOINT.save(
            storage,
            &NominationCheckPoint {
                start_block: None,
                tasks_advancement: 0,
            },
        )?;
        Ok(())
    }
    pub(crate) fn notify_task_created(
        &self,
        storage: &mut dyn Storage,
        env: &Env,
        _config: &Config,
        tasks_advancement: Option<u64>,
    ) -> Result<(), ContractError> {
        //Setting new checkpoint on task advancement
        AGENT_NOMINATION_CHECKPOINT.update(
            storage,
            |mut checkpoint| -> Result<_, ContractError> {
                if checkpoint.start_block.is_none() {
                    checkpoint.start_block = Some(env.block.height)
                }
                Ok(NominationCheckPoint {
                    start_block: checkpoint.start_block,
                    tasks_advancement: checkpoint.tasks_advancement
                        + tasks_advancement.unwrap_or(1),
                })
            },
        )?;

        Ok(())
    }
    pub(crate) fn notify_task_completed(
        &self,
        storage: &mut dyn Storage,
        env: &Env,
        agent_id: Addr,
        is_block_slot_task: bool,
    ) -> Result<(), ContractError> {
        agent_map().update(storage, agent_id.clone().as_bytes(), |old| match old {
            Some(value) => {
                let mut agent = value.1;
                if is_block_slot_task {
                    agent.completed_block_tasks += 1;
                } else {
                    agent.completed_cron_tasks += 1;
                }
                agent.last_executed_slot = env.block.height;
                Ok((agent_id, agent))
            }
            None => Err(ContractError::AgentNotRegistered {}),
        })?;
        Ok(())
    }

    pub(crate) fn cleanup(
        &self,
        storage: &mut dyn Storage,
        env: &Env,
        config: &Config,
    ) -> Result<Vec<Addr>, ContractError> {
        let active_agents = self.agents_active(storage)?;
        let block_height = env.block.height;
        let total_remove_agents: usize = active_agents.len();
        let mut removed_agents = Vec::new();

        for (agent_id, _) in active_agents {
            let skip =
                (config.min_active_reserve as usize) >= total_remove_agents - removed_agents.len();
            if !skip {
                let agent = self.get_agent(storage, &agent_id)?.unwrap();
                if block_height > agent.last_executed_slot + config.max_slot_passover {
                    removed_agents.push(agent_id.clone());
                }
            }
        }
        Ok(removed_agents)
    }
    fn max_nomination_index(
        &self,
        cfg: &Config,
        env: &Env,
        checkpoint: &NominationCheckPoint,
    ) -> Result<Option<u64>, ContractError> {
        let block_height = env.block.height;
        let agents_by_tasks_created = checkpoint.tasks_advancement / cfg.min_tasks_per_agent;
        let agents_by_height = checkpoint.start_block.map_or(0, |start_height| {
            (block_height - start_height) / cfg.agent_nomination_block_duration as u64
        });

        let agents_to_pass = min(agents_by_tasks_created, agents_by_height);
        if agents_to_pass == 0 {
            Ok(None)
        } else {
            Ok(Some(agents_to_pass - 1))
        }
    }

    pub(crate) fn get_available_tasks(
        &self,
        storage: &dyn Storage,
        agent_id: &Addr,
        slots: (u64, u64),
    ) -> Result<(u64, u64), ContractError> {
        if slots == (0, 0) {
            return Ok(slots);
        }
        let (block_slots, cron_slots) = slots;

        let equalizer = |inner: &mut Vec<(Addr, Agent)>,
                         slot_type: SlotType,
                         total_tasks: u64|
         -> Result<u64, ContractError> {
            if total_tasks < 1 {
                return Ok(u64::default());
            }
            let ordering = |left: &Agent,
                            right: &Agent,
                            slot_type: &SlotType|
             -> Option<std::cmp::Ordering> {
                match slot_type {
                    SlotType::Block => {
                        let lr: u128 =
                            format!("{}{}", left.last_executed_slot, left.completed_block_tasks)
                                .parse()
                                .unwrap();
                        let rl: u128 = format!(
                            "{}{}",
                            right.last_executed_slot, right.completed_block_tasks
                        )
                        .parse()
                        .unwrap();

                        lr.partial_cmp(&rl)
                    }
                    SlotType::Cron => {
                        let lr: u128 =
                            format!("{}{}", left.last_executed_slot, left.completed_cron_tasks)
                                .parse()
                                .unwrap();
                        let rl: u128 =
                            format!("{}{}", right.last_executed_slot, right.completed_cron_tasks)
                                .parse()
                                .unwrap();
                        lr.partial_cmp(&rl)
                    }
                }
            };
            //This sort is unstable (i.e., may reorder equal elements), in-place (i.e., does not allocate),
            //and O(n log n) worst-case.
            //It is typically faster than stable sorting, except in a few special cases,
            //e.g., when the slice consists of several concatenated sorted sequences.
            inner.sort_unstable_by(|left, right| ordering(&left.1, &right.1, &slot_type).unwrap());

            let active_total = inner.len() as u64;

            let agent_position = inner.iter().position(|a| agent_id == &a.0).ok_or(
                ContractError::AgentNotActive {
                    addr: agent_id.clone(),
                },
            )? as u64;

            if total_tasks <= active_total {
                let agent_tasks_total = 1u64
                    .saturating_sub(agent_position.saturating_sub(total_tasks.saturating_sub(1)));
                return Ok(agent_tasks_total);
            }

            let leftover = total_tasks % active_total;
            let mut extra = 0u64;
            if leftover > 0 {
                extra =
                    1u64.saturating_sub(agent_position.saturating_sub(leftover.saturating_sub(1)));
            }
            let agent_tasks_total = total_tasks.saturating_div(active_total) + extra;

            Ok(agent_tasks_total)
        };

        let mut active = self.agents_active(storage)?;
        let total_block_tasks = equalizer(&mut active, SlotType::Block, block_slots)?;

        let total_cron_tasks = equalizer(&mut active, SlotType::Cron, cron_slots)?;

        Ok((total_block_tasks, total_cron_tasks))
    }
}
