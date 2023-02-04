use cosmwasm_std::{Addr, Timestamp, Uint64};
use croncat_sdk_tasks::types::{Boundary, Config, Task};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

pub const CONFIG: Item<Config> = Item::new("config");

/// Total amount of tasks
pub const TASKS_TOTAL: Item<u64> = Item::new("tasks_total");

/// Timestamps can be grouped into slot buckets (1-60 second buckets) for easier agent handling
pub const TIME_SLOTS: Map<u64, Vec<Vec<u8>>> = Map::new("time_slots");

/// Block slots allow for grouping of tasks at a specific block height,
/// this is done instead of forcing a block height into a range of timestamps for reliability
pub const BLOCK_SLOTS: Map<u64, Vec<Vec<u8>>> = Map::new("block_slots");

/// Last task creation timestamp
pub const LAST_TASK_CREATION: Item<Timestamp> = Item::new("last_task_creation");

// TODO: make IndexedMap's const as soon as cw_storage_plus new version arrives
pub fn tasks_map<'a>() -> IndexedMap<'a, &'a [u8], Task, TaskIndexes<'a>> {
    let indexes = TaskIndexes {
        owner: MultiIndex::new(owner_idx, "tasks", "tasks__owner"),
        evented: MultiIndex::new(evented_idx, "tasks", "tasks__evented"),
    };
    IndexedMap::new("tasks", indexes)
}

pub struct TaskIndexes<'a> {
    pub owner: MultiIndex<'a, Addr, Task, Addr>,
    pub evented: MultiIndex<'a, u64, Task, u64>,
}

pub fn owner_idx(_pk: &[u8], d: &Task) -> Addr {
    d.owner_addr.clone()
}

/// For filtering to tasks with queries (requiring 'check_result') that are also grouped by boundary (if any)
pub fn evented_idx(_pk: &[u8], d: &Task) -> u64 {
    if d.is_evented() && d.clone().boundary.is_block() {
        let v = match d.boundary.clone() {
            Boundary::Height(h) => {
                h.start.unwrap_or(Uint64::zero()).into()
            },
            _ => u64::default()
        };
        return v;
    }
    u64::default()
}

impl<'a> IndexList<Task> for TaskIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Task>> + '_> {
        let v: Vec<&dyn Index<Task>> = vec![&self.owner, &self.evented];
        Box::new(v.into_iter())
    }
}
