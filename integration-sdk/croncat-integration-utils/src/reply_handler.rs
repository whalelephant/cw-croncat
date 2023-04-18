use crate::error::CronCatContractError;
use crate::{CronCatTaskExecutionInfo, REPLY_CRONCAT_TASK_CREATION};
use cosmwasm_std::{Binary, Reply, Uint64};
use cw_utils::parse_reply_execute_data;

/// Reply handler when a contract calls [`create_task`](croncat_sdk_tasks::msg::TasksExecuteMsg::CreateTask).
/// This will handle [`reply_always`](cosmwasm_std::ReplyOn::Always) covering success and failure.
pub fn reply_handle_task_creation(
    msg: Reply,
) -> Result<(CronCatTaskExecutionInfo, Binary), CronCatContractError> {
    if msg.clone().result.into_result().is_err() {
        return Err(CronCatContractError::ReplyError {
            reply_id: REPLY_CRONCAT_TASK_CREATION.into(),
        });
    }

    let msg_parsed = parse_reply_execute_data(msg);
    let msg_binary = msg_parsed.unwrap().data.unwrap();

    let created_task_info_res = serde_json_wasm::from_slice(msg_binary.as_slice());

    if created_task_info_res.is_err() {
        return Err(CronCatContractError::ReplyError {
            reply_id: Uint64::from(REPLY_CRONCAT_TASK_CREATION),
        });
    }

    let created_task_info: CronCatTaskExecutionInfo = created_task_info_res.unwrap();

    // We return the newly-created task details
    // in your contract's state if you wish.
    // Please see the create-task-handle-tick example for info:
    // https://github.com/CronCats/cw-purrbox/tree/main/contracts
    Ok((created_task_info, msg_binary))
}
