use crate::error::CronCatContractError;
use crate::types::HandleIncomingTaskParams;
use cosmwasm_std::{Addr, Env, MessageInfo, QuerierWrapper};
use croncat_sdk_factory::state::CONTRACT_ADDRS;
use croncat_sdk_manager::state::LAST_TASK_EXECUTION_INFO;
use croncat_sdk_tasks::types::TaskExecutionInfo;

/// Handles and validates an incoming CronCat task
/// Specifically, it checks:
/// - Sender is a sanctioned CronCat manager contract (CronCat factory knows the manager contract addresses and versions.)
/// - We're in the same block and transaction index as the latest executed transaction. In other words, this task is happening in an atomic, synchronous transaction, and this invocation is the result of a cross-contract call (message or submessage) from a manager contract. (Note: you can disable this check via the `disable_sync_check` field of `custom_validation` if, for instance, you're doing IBC calls where the execution is asynchronous, spanning blocks.)
/// - The owner of the task that just called is the calling contract (Note: this can be changed by setting the `expected_owner` field in `custom_validation`. If unset, it will default to this contract.)
/// For contracts storing task hashes, you can take the [`TaskExecutionInfo`](croncat_sdk_tasks::types::TaskExecutionInfo) returned and check it against state.
pub fn handle_incoming_task(
    querier: &QuerierWrapper,
    env: Env,
    info: MessageInfo,
    croncat_factory_address: Addr,
    custom_validation: Option<HandleIncomingTaskParams>,
) -> Result<TaskExecutionInfo, CronCatContractError> {
    // First we'll create helper vars addressing any custom validation
    let HandleIncomingTaskParams {
        disable_sync_check,
        disable_owner_check,
        expected_owner,
    } = custom_validation.unwrap_or_default();

    // If a custom owner is specified, use it. Otherwise, use this contract.
    let owner = expected_owner.unwrap_or(env.contract.address);
    let sender = info.sender;

    // We want to confirm this comes from a sanctioned, CronCat manager
    // contract, which we'll do when we query the factory a bit later
    // This does an efficient query to the sender contract (which may or may not be a sanction manager, which comes later)

    // Pertinent info containing, among other things, the task version
    let latest_task_execution: TaskExecutionInfo = LAST_TASK_EXECUTION_INFO
        .query(querier, sender.clone())
        .map_err(|_| CronCatContractError::LatestTaskInfoFailed {
            manager_addr: sender.clone(),
        })?;

    // We turn (for example) "0.1" into [0, 1] so we can query the factory with this value and the contract name ("manager")
    let versions = latest_task_execution
        .version
        .split('.')
        .map(|v| -> u8 { v.parse().unwrap() })
        .collect::<Vec<u8>>();

    let sanctioned_manager_res: Option<Addr> = CONTRACT_ADDRS.query(
        querier,
        croncat_factory_address,
        ("manager", versions.as_slice()),
    )?;

    if sanctioned_manager_res.is_none() {
        return Err(CronCatContractError::FactoryManagerQueryFailed {
            manager_addr: sender,
            version: latest_task_execution.version,
        });
    }

    let sanctioned_manager_address = sanctioned_manager_res.unwrap();

    // If the sender and the sanctioned manager address differ,
    // then this isn't being called by CronCat
    if sanctioned_manager_address != sender {
        return Err(CronCatContractError::UnsanctionedInvocation {
            manager_addr: sender,
            version: latest_task_execution.version,
        });
    }

    // If this method is called normally (with disable_sync_check defaulting to false)
    // This will check for synchronous invocation from the CronCat manager.
    // This method can be called, ignoring this check by setting it to `true`.
    if !disable_sync_check {
        // Require that this is both in the same block…
        let is_same_block_bool = env.block.height == latest_task_execution.block_height;
        // …and the same transaction index, meaning we're in the
        // middle of a cross-contract call from a sanctioned
        // CronCat manager contract.
        let is_same_tx_id_bool = env.transaction == latest_task_execution.tx_info;

        if !is_same_block_bool || !is_same_tx_id_bool {
            return Err(CronCatContractError::NotSameBlockTxIndex {});
        }
    }

    // Last, we check if the task creator is this contract, ensuring
    // this invocation hasn't happened from someone else's task.
    // In cases where that's too restrictive, you may specify
    if !disable_owner_check && latest_task_execution.owner_addr != owner {
        return Err(CronCatContractError::WrongTaskOwner {
            expected_owner: owner,
        });
    }

    Ok(latest_task_execution)
}
