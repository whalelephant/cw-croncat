#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

#[cfg(test)]
mod tests;

/// Reply ID
/// "croncat" to hex, remove letters
/// dtool s2h croncat | sed 's/0x//' | sed 's/[^0-9]*//g'
pub const REPLY_CRONCAT_TASK_CREATION: u64 = 637266636174;

pub mod error;
pub mod handle_incoming_task;
pub mod reply_handler;
pub mod task_creation;
pub mod types;

pub use croncat_sdk_tasks::types::Action as CronCatAction;
pub use croncat_sdk_tasks::types::Boundary as CronCatBoundary;
pub use croncat_sdk_tasks::types::BoundaryHeight as CronCatBoundaryHeight;
pub use croncat_sdk_tasks::types::BoundaryTime as CronCatBoundaryTime;
pub use croncat_sdk_tasks::types::CosmosQuery as CronCatCosmosQuery;
pub use croncat_sdk_tasks::types::CroncatQuery;
pub use croncat_sdk_tasks::types::Interval as CronCatInterval;
pub use croncat_sdk_tasks::types::TaskExecutionInfo as CronCatTaskExecutionInfo;
pub use croncat_sdk_tasks::types::TaskRequest as CronCatTaskRequest;
pub use croncat_sdk_tasks::types::Transform as CronCatTransform;

pub use croncat_mod_generic::types::PathToValue as CronCatPathToValue;
pub use croncat_mod_generic::types::ValueIndex as CronCatValueIndex;

/// Tasks contract name
pub const TASKS_NAME: &str = "tasks";
/// Manager contract name
pub const MANAGER_NAME: &str = "manager";
/// Agents contract name
pub const AGENTS_NAME: &str = "agents";
