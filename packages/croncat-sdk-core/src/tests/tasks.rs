use cosmwasm_std::to_binary;
use cosmwasm_std::Addr;
use cosmwasm_std::StdError;
use cosmwasm_std::WasmMsg;

use crate::internal_messages::tasks::{RemoveTaskByManager, TasksRescheduleTask};
use crate::internal_messages::tasks::{RescheduleTaskMsg, TasksRemoveTaskByManager};

#[test]
fn tasks_remove_task_by_manager() -> Result<(), StdError> {
    let tasks_remove = TasksRemoveTaskByManager {
        task_hash: "23743450d67e0182ac1c2ace859151e92123bb8b4e3a490a2c0ff8a7b01b0391".into(),
    };

    let msg = tasks_remove.clone().into_binary()?;
    assert_eq!(
        msg,
        to_binary(&RemoveTaskByManager::RemoveTaskByManager(
            tasks_remove.clone()
        ))?
    );

    let cosmos_msg = tasks_remove.into_cosmos_msg(Addr::unchecked("addr"))?;
    assert_eq!(
        cosmos_msg,
        WasmMsg::Execute {
            contract_addr: "addr".into(),
            msg,
            funds: vec![],
        }
        .into()
    );

    Ok(())
}

#[test]
fn tasks_reschedule_task() -> Result<(), StdError> {
    let task_reschedule = TasksRescheduleTask {
        task_hash: "23743450d67e0182ac1c2ace859151e92123bb8b4e3a490a2c0ff8a7b01b0391".into(),
    };

    let msg = task_reschedule.clone().into_binary()?;
    assert_eq!(
        msg,
        to_binary(&RescheduleTaskMsg::RescheduleTask(task_reschedule.clone()))?
    );

    let cosmos_msg = task_reschedule.into_cosmos_msg(Addr::unchecked("addr"))?;
    assert_eq!(
        cosmos_msg,
        WasmMsg::Execute {
            contract_addr: "addr".into(),
            msg,
            funds: vec![],
        }
        .into()
    );

    Ok(())
}
