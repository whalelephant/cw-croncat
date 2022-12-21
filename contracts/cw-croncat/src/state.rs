use crate::{balancer::RoundRobinBalancer, ContractError};
use cosmwasm_std::{Addr, Deps, StdResult, Storage, Timestamp};
use cw2::ContractVersion;
use cw20::Cw20CoinVerified;
use cw_storage_plus::{Deque, Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::helpers::Task;
use cw_croncat_core::{
    query::CroncatQuerier,
    types::{Agent, GasPrice, GenericBalance, SlotType},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    // Runtime
    pub paused: bool,
    pub owner_id: Addr,

    // Agent management
    // The minimum number of tasks per agent
    // Example: 10
    // Explanation: For every 1 agent, 10 tasks per slot are available.
    // NOTE: Caveat, when there are odd number of tasks or agents, the overflow will be available to first-come, first-serve. This doesn't negate the possibility of a failed txn from race case choosing winner inside a block.
    // NOTE: The overflow will be adjusted to be handled by sweeper in next implementation.
    pub min_tasks_per_agent: u64,
    pub agent_active_indices: Vec<(SlotType, u32, u32)>,
    // How many slots an agent can miss before being removed from the active queue
    pub agents_eject_threshold: u64,
    // The duration a prospective agent has to nominate themselves.
    // When a task is created such that a new agent can join,
    // The agent at the zeroth index of the pending agent queue has this time to nominate
    // The agent at the first index has twice this time to nominate (which would remove the former agent from the pending queue)
    // Value is in seconds
    pub agent_nomination_duration: u16,
    pub cw_rules_addr: Addr,

    // Economics
    pub agent_fee: u64,
    pub gas_price: GasPrice,
    pub gas_base_fee: u64,
    pub gas_action_fee: u64,
    pub gas_query_fee: u64,
    pub gas_wasm_query_fee: u64,
    pub proxy_callback_gas: u32,
    pub slot_granularity_time: u64,

    // Treasury
    // pub treasury_id: Option<Addr>,
    pub cw20_whitelist: Vec<Addr>, // TODO: Consider fee structure for whitelisted CW20s
    pub native_denom: String,
    pub available_balance: GenericBalance, // tasks + rewards balances
    pub staked_balance: GenericBalance, // surplus that is temporary staking (to be used in conjunction with external treasury)

    // The default amount of tasks to query
    pub limit: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct QueueItem {
    pub contract_addr: Option<Addr>,
    // This is used to track disjointed callbacks
    // could help scheduling multiple calls across txns
    // could help for IBC non-block bound txns
    // not used yet, need more discover
    // pub prev_idx: Option<u64>,

    // counter of actions helps track what type of action it is
    pub action_idx: u64,
    pub task_hash: Option<Vec<u8>>,
    pub task_is_extra: Option<bool>,
    pub agent_id: Option<Addr>,
    pub failure: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaskInfo {
    pub task: Task,
    pub task_hash: Vec<u8>,
    pub task_is_extra: Option<bool>,
    pub agent_id: Addr,
    pub slot_kind: SlotType,
}

pub struct TaskIndexes<'a> {
    pub owner: MultiIndex<'a, Addr, Task, Addr>,
}

impl<'a> IndexList<Task> for TaskIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Task>> + '_> {
        let v: Vec<&dyn Index<Task>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn token_owner_idx(_pk: &[u8], d: &Task) -> Addr {
    d.owner_id.clone()
}

/// ----------------------------------------------------------------
/// Tasks Storage
/// ----------------------------------------------------------------
pub struct CwCroncat<'a> {
    pub config: Item<'a, Config>,

    pub agents: Map<'a, &'a Addr, Agent>,
    // TODO: Assess if diff store structure is needed for these:
    pub agent_active_queue: Item<'a, Vec<Addr>>,
    pub agent_pending_queue: Deque<'a, Addr>,

    // REF: https://github.com/CosmWasm/cw-plus/tree/main/packages/storage-plus#indexedmap
    pub tasks: IndexedMap<'a, &'a [u8], Task, TaskIndexes<'a>>,
    pub task_total: Item<'a, u64>,

    /// Timestamps can be grouped into slot buckets (1-60 second buckets) for easier agent handling
    pub time_slots: Map<'a, u64, Vec<Vec<u8>>>,
    /// Block slots allow for grouping of tasks at a specific block height,
    /// this is done instead of forcing a block height into a range of timestamps for reliability
    pub block_slots: Map<'a, u64, Vec<Vec<u8>>>,

    pub tasks_with_queries: IndexedMap<'a, &'a [u8], Task, TaskIndexes<'a>>,
    pub tasks_with_queries_total: Item<'a, u64>,

    /// Time and block based maps by the corresponding task hash
    pub time_map_queries: Map<'a, &'a [u8], u64>,
    pub block_map_queries: Map<'a, &'a [u8], u64>,

    /// Reply Queue
    /// Keeping ordered sub messages & reply id's
    pub reply_queue: Map<'a, u64, QueueItem>,
    pub reply_index: Item<'a, u64>,

    // This is a timestamp that's updated when a new task is added such that
    // the agent/task ratio allows for another agent to join.
    // Once an agent joins, fulfilling the need, this value changes to None
    pub agent_nomination_begin_time: Item<'a, Option<Timestamp>>,

    pub balancer: RoundRobinBalancer,
    pub balances: Map<'a, &'a Addr, Vec<Cw20CoinVerified>>,
}

impl Default for CwCroncat<'static> {
    fn default() -> Self {
        Self::new(
            "tasks",
            "tasks_with_queries",
            "tasks__owner",
            "tasks_with_queries__owner",
        )
    }
}

impl<'a> CwCroncat<'a> {
    fn new(
        tasks_key: &'a str,
        tasks_with_queries_key: &'a str,
        tasks_owner_key: &'a str,
        tasks_with_queries_owner_key: &'a str,
    ) -> Self {
        let indexes = TaskIndexes {
            owner: MultiIndex::new(token_owner_idx, tasks_key, tasks_owner_key),
        };
        let indexes_queries = TaskIndexes {
            owner: MultiIndex::new(
                token_owner_idx,
                tasks_with_queries_key,
                tasks_with_queries_owner_key,
            ),
        };
        Self {
            config: Item::new("config"),
            agents: Map::new("agents"),
            agent_active_queue: Item::new("agent_active_queue"),
            agent_pending_queue: Deque::new("agent_pending_queue"),
            tasks: IndexedMap::new(tasks_key, indexes),
            task_total: Item::new("task_total"),
            tasks_with_queries: IndexedMap::new(tasks_with_queries_key, indexes_queries),
            tasks_with_queries_total: Item::new("tasks_with_queries_total"),
            time_slots: Map::new("time_slots"),
            block_slots: Map::new("block_slots"),
            time_map_queries: Map::new("time_slots_queries"),
            block_map_queries: Map::new("block_slots_queries"),
            reply_queue: Map::new("reply_queue"),
            reply_index: Item::new("reply_index"),
            agent_nomination_begin_time: Item::new("agent_nomination_begin_time"),
            balancer: RoundRobinBalancer::default(),
            balances: Map::new("balances"),
        }
    }

    pub fn task_total(&self, storage: &dyn Storage) -> StdResult<u64> {
        self.task_total.load(storage)
    }

    pub fn increment_tasks(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.task_total(storage)? + 1;
        self.task_total.save(storage, &val)?;
        Ok(val)
    }

    pub fn decrement_tasks(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.task_total(storage)? - 1;
        self.task_total.save(storage, &val)?;
        Ok(val)
    }

    pub fn increment_tasks_with_queries(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self
            .tasks_with_queries_total
            .update(storage, |total| -> StdResult<u64> { Ok(total + 1) })?;
        Ok(val)
    }

    pub(crate) fn rq_next_id(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.reply_index.load(storage)? + 1)
    }

    pub(crate) fn rq_push(&self, storage: &mut dyn Storage, item: QueueItem) -> StdResult<u64> {
        let idx = self.reply_index.load(storage)? + 1;
        self.reply_index.save(storage, &idx)?;
        self.reply_queue
            .update(storage, idx, |_d| -> StdResult<QueueItem> { Ok(item) })?;
        Ok(idx)
    }

    pub(crate) fn rq_remove(&self, storage: &mut dyn Storage, idx: u64) {
        self.reply_queue.remove(storage, idx);
    }

    pub(crate) fn rq_update_rq_item(
        &self,
        storage: &mut dyn Storage,
        idx: u64,
        failure: Option<String>,
    ) -> Result<QueueItem, ContractError> {
        self.reply_queue.update(storage, idx, |rq| {
            let mut rq = rq.ok_or(ContractError::UnknownReplyID {})?;
            // if first fails it means whole thing failed
            // for cases where we stop task on failure
            if rq.failure.is_none() {
                rq.failure = failure;
            }
            rq.action_idx += 1;
            Ok(rq)
        })
    }

    pub(crate) fn get_task_by_hash(
        &self,
        storage: &dyn Storage,
        task_hash: &[u8],
    ) -> Result<Task, ContractError> {
        let some_task = self.tasks.may_load(storage, task_hash)?;
        if let Some(task) = some_task {
            Ok(task)
        } else {
            self.tasks_with_queries
                .may_load(storage, task_hash)?
                .ok_or(ContractError::NoTaskFound {})
        }
    }
    pub(crate) fn query_contract_info(
        &self,
        deps: Deps,
        contract_address: String,
    ) -> StdResult<ContractVersion> {
        let querier = CroncatQuerier::new(&deps.querier);
        let res: ContractVersion = querier.query_contract_info(contract_address)?;
        Ok(res)
    }
}
