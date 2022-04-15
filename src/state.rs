use cosmwasm_std::{Addr, Coin, StdResult, Storage};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::agent::Agent;
use crate::helpers::GenericBalance;
use crate::tasks::Task;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    // Runtime
    pub paused: bool,
    pub owner_id: Addr,

    // Agent management
    // The ratio of tasks to agents, where index 0 is agents, index 1 is tasks
    // Example: [1, 10]
    // Explanation: For every 1 agent, 10 tasks per slot are available.
    // NOTE: Caveat, when there are odd number of tasks or agents, the overflow will be available to first-come first-serve. This doesnt negate the possibility of a failed txn from race case choosing winner inside a block.
    // NOTE: The overflow will be adjusted to be handled by sweeper in next implementation.
    pub agent_task_ratio: [u64; 2],
    pub agent_active_index: u64,
    pub agents_eject_threshold: u64,

    // Economics
    pub agent_fee: Coin,
    pub gas_price: u32,
    pub proxy_callback_gas: u32,
    pub slot_granularity: u64,

    // Treasury
    // pub treasury_id: Option<Addr>,
    pub cw20_whitelist: Vec<Addr>, // TODO: Consider fee structure for whitelisted CW20s
    pub native_denom: String,
    pub available_balance: GenericBalance, // tasks + rewards balances
    pub staked_balance: GenericBalance, // surplus that is temporary staking (to be used in conjunction with external treasury)
}

// TODO: Deprecate all instances of USE - moving to declared lifetime
/// ----------------------------------------------------------------
pub const CONFIG: Item<Config> = Item::new("config");
pub const AGENTS: Map<Addr, Agent> = Map::new("agents");
pub const AGENTS_ACTIVE_QUEUE: Item<Vec<Addr>> = Item::new("agent_active_queue");
pub const AGENTS_PENDING_QUEUE: Item<Vec<Addr>> = Item::new("agent_pending_queue");
pub const TASKS: Map<Vec<u8>, Task> = Map::new("tasks");
pub const TASK_OWNERS: Map<Addr, Vec<Vec<u8>>> = Map::new("task_owners");
pub const TIME_SLOTS: Map<u64, Vec<Vec<u8>>> = Map::new("time_slots");
pub const BLOCK_SLOTS: Map<u64, Vec<Vec<u8>>> = Map::new("block_slots");
// END DEPRECATE SECTION
/// ----------------------------------------------------------------

pub struct TaskIndexes<'a> {
    pub owner: MultiIndex<'a, Addr, Task, Addr>,
}

impl<'a> IndexList<Task> for TaskIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Task>> + '_> {
        let v: Vec<&dyn Index<Task>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn token_owner_idx(d: &Task) -> Addr {
    d.owner_id.clone()
}

/// ----------------------------------------------------------------
/// Tasks Storage
/// ----------------------------------------------------------------
pub struct STORE<'a> {
    pub config: Item<'a, Config>,

    pub agents: Map<'a, Addr, Agent>,
    // TODO: Assess if diff store structure is needed for these:
    pub agent_active_queue: Item<'a, Vec<Addr>>,
    pub agent_pending_queue: Item<'a, Vec<Addr>>,

    // REF: https://github.com/CosmWasm/cw-plus/tree/main/packages/storage-plus#indexedmap
    pub tasks: IndexedMap<'a, Vec<u8>, Task, TaskIndexes<'a>>,
    pub task_total: Item<'a, u64>,

    /// Timestamps can be grouped into slot buckets (1-60 second buckets) for easier agent handling
    pub time_slots: Map<'a, u64, Vec<Vec<u8>>>,
    /// Block slots allow for grouping of tasks at a specific block height,
    /// this is done instead of forcing a block height into a range of timestamps for reliability
    pub block_slots: Map<'a, u64, Vec<Vec<u8>>>,
}

impl Default for STORE<'static> {
    fn default() -> Self {
        Self::new(
            "tasks",
            "tasks__owner",
        )
    }
}

impl<'a> STORE<'a> {
    fn new(
        tasks_key: &'a str,
        tasks_owner_key: &'a str,
    ) -> Self {
        let indexes = TaskIndexes {
            owner: MultiIndex::new(token_owner_idx, tasks_key, tasks_owner_key),
        };
        Self {
            config: Item::new("config"),
            agents: Map::new("agents"),
            agent_active_queue: Item::new("agent_active_queue"),
            agent_pending_queue: Item::new("agent_pending_queue"),
            tasks: IndexedMap::new(tasks_key, indexes),
            task_total: Item::new("task_total"),
            time_slots: Map::new("time_slots"),
            block_slots: Map::new("block_slots"),
        }
    }

    pub fn task_total(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.task_total.may_load(storage)?.unwrap_or_default())
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ContractError;
    use crate::slots::{Boundary, Interval};
    use cosmwasm_std::testing::MockStorage;
    use cosmwasm_std::Order;
    use cosmwasm_std::{coins, BankMsg, CosmosMsg, StdResult};
    use cw20::Balance;

    #[test]
    fn check_task_storage_structure() -> StdResult<()> {
        let mut storage = MockStorage::new();
        let store = STORE::default();

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

        // -------------------------

        // create the task
        let res = store
            .tasks
            .update(&mut storage, task.to_hash_vec(), |old| match old {
                Some(_) => Err(ContractError::Unauthorized {}),
                None => Ok(task),
            });
        println!("resssssss {:?}", res);

        // -------------------------

        let task_ids_by_owner: Vec<String> = store
            .tasks
            .idx
            .owner
            .prefix(Addr::unchecked("nobody".to_string()))
            .keys(&mut storage, None, None, Order::Ascending)
            .take(5)
            .map(|x| x.map(|addr| addr.to_string()))
            .collect::<StdResult<Vec<_>>>()?;
        println!("task_ids_by_ownertask_ids_by_owner {:?}", task_ids_by_owner);

        // -------------------------

        let all_tasks: StdResult<Vec<String>> = store
            .tasks
            .range(&mut storage, None, None, Order::Ascending)
            .take(10)
            .map(|x| x.map(|(_, task)| task.to_hash()))
            .collect();
        println!("all_tasks {:?}", all_tasks);

        // -------------------------

        let task_id = "2e87eb9d9dd92e5a903eacb23ce270676e80727bea1a38b40646be08026d05bc"
            .to_string()
            .into_bytes();
        let task = store.tasks.load(&mut storage, task_id)?;
        println!("tasktasktasktasktask {:?}", task);

        // assert!(false);
        Ok(())
    }

    // TODO: Setup test for range / Ordered time slots
}
