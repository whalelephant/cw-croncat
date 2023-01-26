#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

pub mod contract;
pub mod distributor;
pub mod error;
pub mod msg;
pub mod state;
#[cfg(test)]
mod tests;
