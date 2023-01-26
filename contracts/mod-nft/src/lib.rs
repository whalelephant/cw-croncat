#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

pub mod contract;
pub mod msg;
#[cfg(test)]
mod tests;
pub mod types;

pub use mod_sdk::error::ModError as ContractError;
