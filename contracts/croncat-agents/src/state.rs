use crate::distributor::*;
use crate::msg::*;
use cosmwasm_std::Addr;
use croncat_sdk_agents::types::Config;
use croncat_sdk_core::hooks::state::*;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};

//Contract config, just the owner address for now, preferably dao
pub const CONFIG: Item<Config> = Item::new("agents_config");
pub const HOOKS: Hooks = Hooks::new("agent_hooks");
pub(crate) const DEFAULT_NOMINATION_BLOCK_DURATION: u16 = 10;
pub(crate) const DEFAULT_MIN_TASKS_PER_AGENT: u64 = 3;
pub(crate) const DEFAULT_MIN_COINS_FOR_AGENT_REGISTRATION: u64 = 200_000;
pub(crate) const DEFAULT_MAX_SLOTS_PASSOVER: u64 = 600;
pub(crate) const DEFAULT_MIN_ACTIVE_RESERVE: u16 = 1;

pub(crate) const AGENT_NOMINATION_CHECKPOINT: Item<NominationCheckPoint> =
    Item::new("agent_nomination_checkpoint");

// Nomination checkpoint so we can delay agent nomination before some task/block advancement
pub(crate) const AGENT_DISTRIBUTOR: AgentDistributor = AgentDistributor::new();

pub(crate) fn agent_map<'a>() -> IndexedMap<'a, &'a [u8], (Addr, Agent), AgentIndexes<'a>> {
    let indexes = AgentIndexes {
        //by_addr: UniqueIndex::new(addr_idx, "agents_by_addr"),
        by_status: MultiIndex::new(status_idx, "agents", "agents_by_status"),
    };
    IndexedMap::new("agents", indexes)
}

pub(crate) struct AgentIndexes<'a> {
    pub by_status: MultiIndex<'a, String, (Addr, Agent), Addr>,
}
// pub(crate) fn addr_idx(d: &(Addr, Agent)) -> Addr {
//     d.0.clone()
// }

pub(crate) fn status_idx(_pk: &[u8], d: &(Addr, Agent)) -> String {
    d.1.status.to_string()
}

impl<'a> IndexList<(Addr, Agent)> for AgentIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<(Addr, Agent)>> + '_> {
        let v: Vec<&dyn Index<(Addr, Agent)>> = vec![&self.by_status];
        Box::new(v.into_iter())
    }
}
