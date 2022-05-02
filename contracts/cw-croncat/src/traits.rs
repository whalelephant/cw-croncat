use cosmwasm_std::Env;
use cw20::Balance;
use cw_croncat_core::types::{Boundary, SlotType};

pub trait GenericBalances {
    fn add_tokens(&mut self, add: Balance);
    fn minus_tokens(&mut self, minus: Balance);
}

pub trait IntervalExt {
    fn next(&self, env: Env, boundary: Boundary) -> (u64, SlotType);
    fn is_valid(&self) -> bool;
}

pub trait TaskHash {
    fn to_hash(&self) -> String;
    fn to_hash_vec(&self) -> Vec<u8>;
}
