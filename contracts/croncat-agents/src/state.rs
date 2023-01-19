use cosmwasm_std::{Addr, Timestamp};
use croncat_sdk_agents::types::Agent;
use croncat_sdk_core::types::Config;
use croncat_sdk_tasks::types::SlotType;
use cw_storage_plus::{Deque, Item, Map};

use crate::balancer::RoundRobinBalancer;

pub const AGENTS: Map<&Addr, Agent> = Map::new("agents");
pub const ACTIVE_AGENTS: Item<Vec<Addr>> = Item::new("agent_active_queue");
pub const PENDING_AGENTS: Deque<Addr> = Deque::new("agent_pending_queue");
/// Contract config, just the owner address for now, preferably dao
pub const CONFIG: Item<Config> = Item::new("config");
pub const AGENT_NOMINATION_BEGIN_TIME: Item<Option<Timestamp>> =
    Item::new("agent_nomination_begin_time");

pub const AGENT_BALANCER_STATS:Item<Vec<(SlotType, u32, u32)>>=Item::new("agent_task_info");
pub const AGENT_BALANCER:RoundRobinBalancer=RoundRobinBalancer::default();