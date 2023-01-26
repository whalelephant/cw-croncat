use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
#[derive(std::hash::Hash)]
#[cw_serde]
pub enum SlotType {
    Block,
    Cron,
}
#[cw_serde]
pub struct TaskInfo {
    pub task_hash: Vec<u8>,
    pub task_is_extra: Option<bool>,
    pub agent_id: Addr,
    pub slot_kind: SlotType,
}
