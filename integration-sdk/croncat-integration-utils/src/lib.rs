#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

#[cfg(test)]
mod tests;

// Reply ID
pub const REPLY_CRONCAT_TASK_CREATION: u64 = 0;

pub mod error;
pub mod handle_incoming_task;
pub mod reply_handler;
pub mod types;
pub mod cosmwasm_storage_helpers;

pub use croncat_sdk_tasks::types::TaskExecutionInfo as CronCatTaskExecutionInfo;
