use cosmwasm_std::Addr;
use croncat_sdk_tasks::types::{Config, Task};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

pub const CONFIG: Item<Config> = Item::new("config");

/// Total amount of tasks without queries
pub const TASKS_TOTAL: Item<u64> = Item::new("tasks_total");

/// Total amount of tasks with queries
pub const TASKS_WITH_QUERIES_TOTAL: Item<u64> = Item::new("tasks_with_queries_total");

/// Timestamps can be grouped into slot buckets (1-60 second buckets) for easier agent handling
pub const TIME_SLOTS: Map<u64, Vec<Vec<u8>>> = Map::new("time_slots");

/// Block slots allow for grouping of tasks at a specific block height,
/// this is done instead of forcing a block height into a range of timestamps for reliability
pub const BLOCK_SLOTS: Map<u64, Vec<Vec<u8>>> = Map::new("block_slots");

/// Time based map by the corresponding task hash
pub const TIME_MAP_QUERIES: Map<&[u8], u64> = Map::new("time_map_queries");

/// Block based map by the corresponding task hash
pub const BLOCK_MAP_QUERIES: Map<&[u8], u64> = Map::new("block_map_queries");

pub fn tasks_map<'a>() -> IndexedMap<'a, &'a [u8], Task, TaskIndexes<'a>> {
    let indexes = TaskIndexes {
        owner: MultiIndex::new(token_owner_idx, "tasks", "tasks__owner"),
    };
    IndexedMap::new("tasks", indexes)
}

pub fn tasks_with_queries_map<'a>() -> IndexedMap<'a, &'a [u8], Task, TaskIndexes<'a>> {
    let indexes = TaskIndexes {
        owner: MultiIndex::new(
            token_owner_idx,
            "tasks_with_queries",
            "tasks_with_queries__owner",
        ),
    };
    IndexedMap::new("tasks_with_queries", indexes)
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
    d.owner_addr.clone()
}
