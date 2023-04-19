use crate::error::CronCatContractError;
use crate::types::{CronCatTaskSubmessageParams, SubMessageReplyType};
use crate::{REPLY_CRONCAT_TASK_CREATION, TASKS_NAME};
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::WasmMsg::Execute;
use cosmwasm_std::{to_binary, Addr, CosmosMsg, MessageInfo, QuerierWrapper, SubMsg};
use croncat_sdk_factory::msg::ContractMetadataResponse;
use croncat_sdk_factory::msg::FactoryQueryMsg::LatestContract;
use croncat_sdk_tasks::msg::TasksExecuteMsg::CreateTask;
use croncat_sdk_tasks::types::TaskRequest;

/// Given the CronCat factory address, returns the proper contract address if it exists.
/// See [TASKS_NAME](crate::TASKS_NAME), [MANAGER_NAME](crate::MANAGER_NAME), and [AGENTS_NAME](crate::AGENTS_NAME)
pub fn get_latest_croncat_contract(
    querier: &QuerierWrapper,
    croncat_factory_address: Addr,
    croncat_contract_name: String,
) -> Result<Addr, CronCatContractError> {
    let query_factory_msg = LatestContract {
        contract_name: croncat_contract_name.clone(),
    };
    let latest_contract_res: ContractMetadataResponse =
        querier.query_wasm_smart(&croncat_factory_address, &query_factory_msg)?;

    // Check validity of result
    if latest_contract_res.metadata.is_none() {
        return Err(CronCatContractError::NoSuchContractOnFactory {
            contract_name: croncat_contract_name,
            factory_addr: croncat_factory_address,
        });
    }

    let tasks_address = latest_contract_res.metadata.unwrap().contract_addr;

    Ok(tasks_address)
}

/// Returns a SubMsg
/// This can be conveniently used when returning a Response
/// where you might handle what happened in the reply entry point.
/// `Ok(Response::new().add_submessage(returned_val))`
pub fn create_croncat_task_submessage(
    querier: &QuerierWrapper,
    info: MessageInfo,
    croncat_factory_address: Addr,
    task: TaskRequest,
    reply_type: Option<CronCatTaskSubmessageParams>,
) -> Result<SubMsg, CronCatContractError> {
    croncat_basic_validation(info.clone())?;
    let wasm_exec_msg =
        create_croncat_task_cosmos_msg(querier, info, croncat_factory_address, task)?;

    // If no reply_type is provided, will use "always"
    let (reply_id, sub_reply_type) = match reply_type {
        None => (REPLY_CRONCAT_TASK_CREATION, SubMessageReplyType::Always),
        Some(params) => (
            params.reply_id.unwrap_or(REPLY_CRONCAT_TASK_CREATION),
            params.reply_type.unwrap_or(SubMessageReplyType::Always),
        ),
    };

    let sub_message = match sub_reply_type {
        SubMessageReplyType::Always => SubMsg::reply_always(wasm_exec_msg, reply_id),
        SubMessageReplyType::OnError => SubMsg::reply_on_error(wasm_exec_msg, reply_id),
        SubMessageReplyType::OnSuccess => SubMsg::reply_on_success(wasm_exec_msg, reply_id),
    };

    Ok(sub_message)
}

/// Returns a CosmosMsg
/// This can be conveniently used when returning a Response
/// `Ok(Response::new().add_message(returned_val))`
pub fn create_croncat_task_message(
    querier: &QuerierWrapper,
    info: MessageInfo,
    croncat_factory_address: Addr,
    task: TaskRequest,
) -> Result<CosmosMsg, CronCatContractError> {
    croncat_basic_validation(info.clone())?;
    let wasm_exec_msg =
        create_croncat_task_cosmos_msg(querier, info, croncat_factory_address, task)?;

    Ok(wasm_exec_msg)
}

/// This returns a CosmosMsg Execute object
/// It's a helper in this crate, but is exposed
/// for external usage as well.
pub fn create_croncat_task_cosmos_msg(
    querier: &QuerierWrapper,
    info: MessageInfo,
    croncat_factory_address: Addr,
    task: TaskRequest,
) -> Result<CosmosMsg, CronCatContractError> {
    let tasks_addr =
        get_latest_croncat_contract(querier, croncat_factory_address, TASKS_NAME.to_string())?;

    Ok(Wasm(Execute {
        contract_addr: String::from(tasks_addr),
        msg: to_binary(&CreateTask {
            task: Box::new(task),
        })?,
        funds: info.funds,
    }))
}

pub fn croncat_basic_validation(info: MessageInfo) -> Result<(), CronCatContractError> {
    // To create a CronCat task you will need to provide funds. All funds sent
    // to this method will be used for task creation
    // Because we cannot detect detailed error information from replies
    // (See CosmWasm Discord: https://discord.com/channels/737637324434833438/737643344712171600/1040920787512725574)
    // We'll add a check here to ensure they've attached funds
    if info.funds.is_empty() {
        return Err(CronCatContractError::TaskCreationNoFunds {});
    }

    Ok(())
}
