use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const MANAGER_ADDR: Item<Addr> = Item::new("manager_addr");
