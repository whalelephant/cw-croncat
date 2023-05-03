use croncat_sdk_tasks::types::TaskExecutionInfo;
use cw_storage_plus::Item;

/// Safe way to export map of the croncat-factory, but avoid any contract imports
/// Contract name with the version to the Addr
pub const LAST_TASK_EXECUTION_INFO: Item<TaskExecutionInfo> = Item::new("last_task_execution_info");
