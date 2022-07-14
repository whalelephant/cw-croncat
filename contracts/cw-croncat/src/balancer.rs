use crate::state::Config;
use crate::state::TaskIndexes;
use crate::{slots, ContractError::AgentNotRegistered};
use cosmwasm_std::Uint64;
use cosmwasm_std::{Addr, Deps, Env, StdError, StdResult, Storage};
use cw_croncat_core::msg::AgentTaskResponse;
use cw_croncat_core::types::{Agent, SlotType, Task};
use cw_storage_plus::IndexedMap;
use cw_storage_plus::Item;

pub trait Balancer<'a> {
    fn next_task(
        &mut self,
        deps: Deps,
        env: Env,
        config: &Item<'a, Config>,
        active_agents: &Item<'a, Vec<Addr>>,
        agent_id: Addr,
        slot_items: (Option<u64>, Option<u64>),
    ) -> StdResult<Option<AgentTaskResponse>>;
}

pub struct RoundRobinBalancer {}

impl<'a> RoundRobinBalancer {
    pub fn new() -> RoundRobinBalancer {
        return RoundRobinBalancer {};
    }
}
impl<'a> Balancer<'a> for RoundRobinBalancer {
    fn next_task(
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
            // TODO: unsure if we can return AgentNotRegistered
            return Err(StdError::GenericErr {
                msg: AgentNotRegistered {}.to_string(),
            });
        }
        let agent_active_indices_config=conf.agent_active_indices;
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

        Ok(Some(AgentTaskResponse {
            num_block_tasks,
            num_cron_tasks,
        }))
    }
}
