use crate::distributor::AgentTaskDistributor;
use crate::msg::*;
use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Deque, Item, Map};

/// Contract config, just the owner address for now, preferably dao
pub const CONFIG: Item<Config> = Item::new("agents_config");
pub const AGENT_NOMINATION_BEGIN_TIME: Item<Option<Timestamp>> =
    Item::new("agent_nomination_begin_time");

pub(crate) const DEFAULT_NOMINATION_DURATION: u16 = 360;
pub(crate) const DEFAULT_MIN_TASKS_PER_AGENT: u64 = 3;

pub const AGENTS: Map<&Addr, Agent> = Map::new("agents");
pub const AGENTS_ACTIVE: Item<Vec<Addr>> = Item::new("agents_active");
pub const AGENTS_PENDING: Deque<Addr> = Deque::new("agents_pending");
pub const AGENT_STATS: Map<&Addr, AgentStats> = Map::new("agent_stats");
pub const AGENT_TASK_DISTRIBUTOR: AgentTaskDistributor = AgentTaskDistributor::new();
