use cosmwasm_std::Addr;

/// This struct may be provided when calling the method [`handle_incoming_task`](crate::handle_incoming_task::handle_incoming_task).
#[derive(Default)]
pub struct HandleIncomingTaskParams {
    /// Disables the check ensuring that the block height and transaction index are the same
    /// If you expect an IBC delay or something asynchronous, disable by setting to true.
    pub disable_sync_check: bool,
    /// By default, we check that the contract receiving the task invocation
    /// must be the owner of the that task. Put another way: someone else's
    /// task isn't invoking our method. You can disable this by setting to true.
    pub disable_owner_check: bool,
    /// If the owner check is enabled, you may specify an alternate expected owner.
    /// Perhaps the task owner isn't this contract, but you know the address.
    /// By default, the validation logic in `handle_incoming_task` checks against the current contract.
    /// If disable_owner_check is true, this value is irrelevant.
    pub expected_owner: Option<Addr>,
}

/// CosmWasm "reply on" types for submessages.
/// See <https://book.cosmwasm.com/actor-model/contract-as-actor.html#sending-submessages>
pub enum SubMessageReplyType {
    Always,
    OnError,
    OnSuccess,
}

/// Extra (optional) parameters when creating a submessage during task creation
pub struct CronCatTaskSubmessageParams {
    /// Defaults to [REPLY_CRONCAT_TASK_CREATION](crate::REPLY_CRONCAT_TASK_CREATION)
    pub reply_id: Option<u64>,
    /// Defaults to [Always](SubMessageReplyType::Always)
    pub reply_type: Option<SubMessageReplyType>,
}
