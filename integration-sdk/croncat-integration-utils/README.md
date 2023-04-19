# CronCat Integration Utilities

This repo contains helper methods for projects adding automation to their smart contracts using CronCat tasks.

Crate: https://crates.io/crates/croncat-integration-utils
Rust docs: https://docs.rs/croncat-integration-utils/latest/croncat_integration_utils

Examples using this crate: https://github.com/CronCats/cw-purrbox

## Task creation

- `get_latest_croncat_contract` — provided the CronCat factory address and a contract name (e.g. "tasks") it'll return the address of the latest contract, or throw a custom error.
- `create_croncat_task_submessage` — this is likely a function that integrating dApps will use frequently. DApps are able to create a `TaskRequest` object (exported as `CronCatTaskRequest`), provide the factory address and optional reply parameters. It will return a Result with a submessage that can be added to the contract's Response, like `Response::new().add_submessage(s)(…`
- `create_croncat_task_message` — same as the submessage one above, except as a message, for integrating dApps that have decided they don't need a reply, for whatever reason they've landed on.
- `create_croncat_task_cosmos_msg` — this function is being used as a helper during the previous calls, but might as well be exported for developers, as doing so can only be helpful. It returns a Result of the Wasm Execute message to create the CronCat task.
- `croncat_basic_validation` — this function can grow in the future if we wish, and only contains one validation, which is ensuring that funds have been attached. Due to the nature of error handling, it feels like we must attempt to do basic validation in order to reduce DevX confusion from integrators.

```rs
pub struct CronCatTaskSubmessageParams {
    /// Defaults to [REPLY_CRONCAT_TASK_CREATION](crate::REPLY_CRONCAT_TASK_CREATION)
    pub reply_id: Option<u64>,
    /// Defaults to [Always](SubMessageReplyType::Always)
    pub reply_type: Option<SubMessageReplyType>,
}
```

The `REPLY_CRONCAT_TASK_CREATION` is set to a large, unique number derived from the word "croncat" to reduce collisions in integrating dApps.

Also, a simple enum for Reply type, which can be specified if desired:

```rs
pub enum SubMessageReplyType {
    Always,
    OnError,
    OnSuccess,
}
```

## Handle reply after task creation

When you create a task using a submessage with replies, you can utilize two functions in your contract's reply entry point.

- `reply_handle_task_creation` — this takes the `msg: Reply` argument and parses it into the more useful `CronCatTaskExecutionInfo`, which is just the `croncat-sdk-tasks`'s `TaskExecutionInfo`. It returns a tuple with the logic in the reply entry point and the binary message data. The latter is useful to include in the reply's Response, like by using `Reply::new().set_data(…`.

## Handle invocation from a task

Methods introduced here:
- `handle_incoming_task` — this will check the version of the task, and confirm (via a factory query) that the sender is a sanctioned manager contract. It also checks that the execution is happening synchronously by confirming via state that the manager saves right before sending the messages invoking it. Finally, it checks that the owner of the task is the receiving contract itself, or else the `expected_owner` provided in the optional parameters.

In this scenario, your contract is getting called, at a method that we expect to be called by CronCat. We want to know if indeed it's CronCat executing your task, and also check that the task's owner is the receiving contract. If you wish to modify these restrictions, you are able to with this struct for additional parameters:

```rs
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
```
