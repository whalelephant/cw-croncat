#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use croncat_sdk_core::internal_messages::agents::AgentOnTaskCreated;
use croncat_sdk_core::internal_messages::manager::{ManagerCreateTaskBalance, ManagerRemoveTask};
use croncat_sdk_core::internal_messages::tasks::{TasksRemoveTaskByManager, TasksRescheduleTask};
use croncat_sdk_tasks::msg::UpdateConfigMsg;
use croncat_sdk_tasks::types::{
    Config, SlotHashesResponse, SlotIdsResponse, SlotTasksTotalResponse, SlotType, Task,
    TaskRequest, TaskResponse,
};
use cw2::{query_contract_info, set_contract_version};
use cw20::Cw20CoinVerified;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::helpers::{
    check_if_sender_is_manager, get_agents_addr, get_manager_addr, remove_task_with_queries,
    remove_task_without_queries, validate_boundary, validate_msg_calculate_usage,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    tasks_map, tasks_with_queries_map, BLOCK_MAP_QUERIES, BLOCK_SLOTS, CONFIG, TASKS_TOTAL,
    TASKS_WITH_QUERIES_TOTAL, TIME_MAP_QUERIES, TIME_SLOTS,
};

const CONTRACT_NAME: &str = "croncat:croncat-tasks";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Default value based on non-wasm operations, wasm ops seem impossible to predict
// TODO: this values based of pre-split, need to recalculate GAS_BASE_FEE
pub(crate) const GAS_BASE_FEE: u64 = 300_000;
pub(crate) const GAS_ACTION_FEE: u64 = 130_000;
pub(crate) const GAS_QUERY_FEE: u64 = 130_000; // Load query module(~61_000) and query after that(~65_000+)
pub(crate) const GAS_LIMIT: u64 = 9_500_000; // 10M is default for juno, but let's make sure we have space for missed gas calculations
pub(crate) const SLOT_GRANULARITY_TIME: u64 = 10_000_000_000; // 10 seconds

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let InstantiateMsg {
        chain_name,
        owner_addr,
        croncat_manager_key,
        croncat_agents_key,
        slot_granularity_time,
        gas_base_fee,
        gas_action_fee,
        gas_query_fee,
        gas_limit,
    } = msg;
    let owner_addr = owner_addr
        .map(|human| deps.api.addr_validate(&human))
        .transpose()?
        .unwrap_or(info.sender.clone());
    let config = Config {
        paused: false,
        chain_name,
        owner_addr,
        croncat_factory_addr: info.sender,
        croncat_manager_key,
        croncat_agents_key,
        slot_granularity_time: slot_granularity_time.unwrap_or(SLOT_GRANULARITY_TIME),
        gas_base_fee: gas_base_fee.unwrap_or(GAS_BASE_FEE),
        gas_action_fee: gas_action_fee.unwrap_or(GAS_ACTION_FEE),
        gas_query_fee: gas_query_fee.unwrap_or(GAS_QUERY_FEE),
        gas_limit: gas_limit.unwrap_or(GAS_LIMIT),
    };
    // Save initializing states
    CONFIG.save(deps.storage, &config)?;
    TASKS_TOTAL.save(deps.storage, &0)?;
    TASKS_WITH_QUERIES_TOTAL.save(deps.storage, &0)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig(msg) => execute_update_config(deps, msg),
        ExecuteMsg::CreateTask { task } => execute_create_task(deps, env, info, *task),
        ExecuteMsg::RemoveTask { task_hash } => execute_remove_task(deps, info, task_hash),
        // Methods for other contracts
        ExecuteMsg::RemoveTaskByManager(remove_task_msg) => {
            execute_remove_task_by_manager(deps, info, remove_task_msg)
        }
        ExecuteMsg::RescheduleTask(reschedule_msg) => {
            execute_reschedule_task(deps, env, info, reschedule_msg)
        }
    }
}

fn execute_update_config(deps: DepsMut, msg: UpdateConfigMsg) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // Destruct so we won't forget to update if if new fields added
    let UpdateConfigMsg {
        paused,
        owner_addr,
        croncat_factory_addr,
        croncat_manager_key,
        croncat_agents_key,
        slot_granularity_time,
        gas_base_fee,
        gas_action_fee,
        gas_query_fee,
        gas_limit,
    } = msg;

    let new_config = Config {
        paused: paused.unwrap_or(config.paused),
        owner_addr: owner_addr
            .map(|human| deps.api.addr_validate(&human))
            .transpose()?
            .unwrap_or(config.owner_addr),
        croncat_factory_addr: croncat_factory_addr
            .map(|human| deps.api.addr_validate(&human))
            .transpose()?
            .unwrap_or(config.croncat_factory_addr),
        chain_name: config.chain_name,
        croncat_manager_key: croncat_manager_key.unwrap_or(config.croncat_manager_key),
        croncat_agents_key: croncat_agents_key.unwrap_or(config.croncat_agents_key),
        slot_granularity_time: slot_granularity_time.unwrap_or(config.slot_granularity_time),
        gas_base_fee: gas_base_fee.unwrap_or(config.gas_base_fee),
        gas_action_fee: gas_action_fee.unwrap_or(config.gas_action_fee),
        gas_query_fee: gas_query_fee.unwrap_or(config.gas_query_fee),
        gas_limit: gas_limit.unwrap_or(config.gas_limit),
    };

    CONFIG.save(deps.storage, &new_config)?;
    Ok(Response::new().add_attribute("action", "update_config"))
}

fn execute_reschedule_task(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    reschedule_msg: TasksRescheduleTask,
) -> Result<Response, ContractError> {
    let task_hash = reschedule_msg.task_hash;
    let config = CONFIG.load(deps.storage)?;
    check_if_sender_is_manager(&deps.querier, &config, &info.sender)?;

    let mut remove_task = None;
    // Check default map
    let (next_id, slot_kind) = if let Some(task) = tasks_map().may_load(deps.storage, &task_hash)? {
        let (next_id, slot_kind) =
            task.interval
                .next(&env, &task.boundary, config.slot_granularity_time);
        if next_id != 0 {
            // Get previous task hashes in slot, add as needed
            let update_vec_data = |d: Option<Vec<Vec<u8>>>| -> StdResult<Vec<Vec<u8>>> {
                match d {
                    // has some data, simply push new hash
                    Some(data) => {
                        let mut s = data;
                        s.push(task_hash);
                        Ok(s)
                    }
                    // No data, push new vec & hash
                    None => Ok(vec![task_hash]),
                }
            };
            // Based on slot kind, put into block or cron slots
            match slot_kind {
                SlotType::Block => {
                    BLOCK_SLOTS.update(deps.storage, next_id, update_vec_data)?;
                }
                SlotType::Cron => {
                    TIME_SLOTS.update(deps.storage, next_id, update_vec_data)?;
                }
            }
        } else {
            remove_task_without_queries(deps.storage, &task_hash, task.boundary.is_block_boundary)?;
            remove_task = Some(ManagerRemoveTask {
                sender: info.sender,
                task_hash,
            });
        }
        (next_id, slot_kind)
    } else if let Some(task) = tasks_with_queries_map().may_load(deps.storage, &task_hash)? {
        let (next_id, slot_kind) =
            task.interval
                .next(&env, &task.boundary, config.slot_granularity_time);
        if next_id != 0 {
            match slot_kind {
                SlotType::Block => {
                    BLOCK_MAP_QUERIES.save(deps.storage, &task_hash, &next_id)?;
                }
                SlotType::Cron => {
                    TIME_MAP_QUERIES.save(deps.storage, &task_hash, &next_id)?;
                }
            }
        } else {
            remove_task_with_queries(deps.storage, &task_hash, task.boundary.is_block_boundary)?;
            remove_task = Some(ManagerRemoveTask {
                sender: info.sender,
                task_hash,
            });
        }
        (next_id, slot_kind)
    } else {
        return Err(ContractError::NoTaskFound {});
    };

    Ok(Response::new()
        .add_attribute("action", "reschedule_task")
        .add_attribute("slot_id", next_id.to_string())
        .add_attribute("slot_kind", slot_kind.to_string())
        .set_data(to_binary(&remove_task)?))
}

fn execute_remove_task_by_manager(
    deps: DepsMut,
    info: MessageInfo,
    remove_task_msg: TasksRemoveTaskByManager,
) -> Result<Response, ContractError> {
    let task_hash = remove_task_msg.task_hash;
    let config = CONFIG.load(deps.storage)?;
    check_if_sender_is_manager(&deps.querier, &config, &info.sender)?;

    if let Some(task) = tasks_map().may_load(deps.storage, &task_hash)? {
        remove_task_without_queries(deps.storage, &task_hash, task.boundary.is_block_boundary)?;
    } else if let Some(task) = tasks_with_queries_map().may_load(deps.storage, &task_hash)? {
        remove_task_with_queries(deps.storage, &task_hash, task.boundary.is_block_boundary)?;
    } else {
        return Err(ContractError::NoTaskFound {});
    }

    Ok(Response::new().add_attribute("action", "remove_task_by_manager"))
}

fn execute_remove_task(
    deps: DepsMut,
    info: MessageInfo,
    task_hash: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.paused {
        return Err(ContractError::Paused {});
    }
    let hash = task_hash.as_bytes();
    if let Some(task) = tasks_map().may_load(deps.storage, hash)? {
        if task.owner_addr != info.sender {
            return Err(ContractError::Unauthorized {});
        }
        remove_task_without_queries(deps.storage, hash, task.boundary.is_block_boundary)?;
    } else if let Some(task) = tasks_with_queries_map().may_load(deps.storage, hash)? {
        if task.owner_addr != info.sender {
            return Err(ContractError::Unauthorized {});
        }
        remove_task_with_queries(deps.storage, hash, task.boundary.is_block_boundary)?;
    } else {
        return Err(ContractError::NoTaskFound {});
    }
    let manager_addr = get_manager_addr(&deps.querier, &config)?;
    let remove_task_msg = ManagerRemoveTask {
        sender: info.sender,
        task_hash: task_hash.into_bytes(),
    }
    .into_cosmos_msg(manager_addr)?;
    Ok(Response::new()
        .add_attribute("action", "remove_task")
        .add_message(remove_task_msg))
}

fn execute_create_task(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    task: TaskRequest,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.paused {
        return Err(ContractError::Paused {});
    }
    let owner_addr = info.sender;

    // Validate boundary and interval
    let boundary = validate_boundary(&env.block, &task.boundary, &task.interval)?;
    if !task.interval.is_valid() {
        return Err(ContractError::InvalidInterval {});
    }

    let amount_for_one_task =
        validate_msg_calculate_usage(deps.api, &task, &env.contract.address, &owner_addr, &config)?;
    if amount_for_one_task.gas > config.gas_limit {
        return Err(ContractError::InvalidGas {});
    }
    let cw20 = task
        .cw20
        .map(|human| {
            StdResult::Ok(Cw20CoinVerified {
                address: deps.api.addr_validate(&human.address)?,
                amount: human.amount,
            })
        })
        .transpose()?;

    let version = query_contract_info(&deps.querier, env.contract.address.as_str())?.version;
    let item = Task {
        owner_addr: owner_addr.clone(),
        interval: task.interval,
        boundary,
        stop_on_fail: task.stop_on_fail,
        amount_for_one_task: amount_for_one_task.clone(),
        actions: task.actions,
        queries: task.queries.unwrap_or_default(),
        transforms: task.transforms.unwrap_or_default(),
        version,
    };
    let hash_prefix = &config.chain_name;
    let hash = item.to_hash(hash_prefix);

    let (next_id, slot_kind) =
        item.interval
            .next(&env, &item.boundary, config.slot_granularity_time);
    if next_id == 0 {
        return Err(ContractError::TaskEnded {});
    }

    let recurring = item.recurring();
    let with_queries = item.with_queries();
    if with_queries {
        match slot_kind {
            SlotType::Block => BLOCK_MAP_QUERIES.save(deps.storage, hash.as_bytes(), &next_id),
            SlotType::Cron => TIME_MAP_QUERIES.save(deps.storage, hash.as_bytes(), &next_id),
        }?;
        TASKS_WITH_QUERIES_TOTAL.update(deps.storage, |amt| -> StdResult<_> { Ok(amt + 1) })?;
        tasks_with_queries_map().update(deps.storage, hash.as_bytes(), |old| match old {
            Some(_) => Err(ContractError::TaskExists {}),
            None => Ok(item),
        })?;
    } else {
        let hash = hash.clone().into_bytes();
        TASKS_TOTAL.update(deps.storage, |amt| -> StdResult<_> { Ok(amt + 1) })?;
        tasks_map().update(deps.storage, &hash, |old| match old {
            Some(_) => Err(ContractError::TaskExists {}),
            None => Ok(item),
        })?;
        // Get previous task hashes in slot, add as needed
        let update_vec_data = |d: Option<Vec<Vec<u8>>>| -> StdResult<Vec<Vec<u8>>> {
            match d {
                // has some data, simply push new hash
                Some(data) => {
                    let mut s = data;
                    s.push(hash);
                    Ok(s)
                }
                // No data, push new vec & hash
                None => Ok(vec![hash]),
            }
        };
        match slot_kind {
            SlotType::Block => {
                BLOCK_SLOTS.update(deps.storage, next_id, update_vec_data)?;
            }
            SlotType::Cron => {
                TIME_SLOTS.update(deps.storage, next_id, update_vec_data)?;
            }
        }
    }

    let manager_addr = get_manager_addr(&deps.querier, &config)?;
    let manager_create_task_balance_msg = ManagerCreateTaskBalance {
        sender: owner_addr,
        task_hash: hash.as_bytes().to_owned(),
        recurring,
        cw20,
        amount_for_one_task,
    }
    .into_cosmos_msg(manager_addr, info.funds)?;

    let agent_addr = get_agents_addr(&deps.querier, &config)?;
    let agent_new_task_msg = AgentOnTaskCreated {
        task_hash:hash.clone()
    }
    .into_cosmos_msg(agent_addr)?;
    Ok(Response::new()
        .set_data(hash.as_bytes())
        .add_attribute("action", "create_task")
        .add_attribute("slot_id", next_id.to_string())
        .add_attribute("slot_kind", slot_kind.to_string())
        .add_attribute("task_hash", hash)
        .add_attribute("with_queries", with_queries.to_string())
        .add_message(manager_create_task_balance_msg)
        .add_message(agent_new_task_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::TasksTotal {} => {
            to_binary(&cosmwasm_std::Uint64::from(TASKS_TOTAL.load(deps.storage)?))
        }
        QueryMsg::TasksWithQueriesTotal {} => to_binary(&cosmwasm_std::Uint64::from(
            TASKS_WITH_QUERIES_TOTAL.load(deps.storage)?,
        )),
        QueryMsg::Tasks { from_index, limit } => to_binary(&query_tasks(deps, from_index, limit)?),
        QueryMsg::TasksWithQueries { from_index, limit } => {
            to_binary(&query_tasks_with_queries(deps, from_index, limit)?)
        }
        QueryMsg::TasksByOwner {
            owner_addr,
            from_index,
            limit,
        } => to_binary(&query_tasks_by_owner(deps, owner_addr, from_index, limit)?),
        QueryMsg::Task { task_hash } => to_binary(&query_task(deps, task_hash)?),
        QueryMsg::TaskHash { task } => to_binary(&query_task_hash(deps, *task)?),
        QueryMsg::SlotHashes { slot } => to_binary(&query_slot_hashes(deps, slot)?),
        QueryMsg::SlotIds { from_index, limit } => {
            to_binary(&query_slot_ids(deps, from_index, limit)?)
        }
        QueryMsg::SlotTasksTotal { offset } => {
            to_binary(&query_slot_tasks_total(deps, env, offset)?)
        }
        QueryMsg::GetCurrentTask {} => to_binary(&query_get_current_task(deps, env)?),
    }
}

fn query_slot_tasks_total(
    deps: Deps,
    env: Env,
    offset: Option<u64>,
) -> StdResult<SlotTasksTotalResponse> {
    if let Some(off) = offset {
        let config = CONFIG.load(deps.storage)?;
        let block_tasks = BLOCK_SLOTS
            .may_load(deps.storage, env.block.height + off)?
            .unwrap_or_default()
            .len() as u64;

        let current_block_ts = env.block.time.nanos();
        let current_block_slot =
            current_block_ts.saturating_sub(current_block_ts % config.slot_granularity_time);
        let cron_tasks = TIME_SLOTS
            .may_load(
                deps.storage,
                current_block_slot + config.slot_granularity_time * off,
            )?
            .unwrap_or_default()
            .len() as u64;
        Ok(SlotTasksTotalResponse {
            block_tasks,
            cron_tasks,
        })
    } else {
        let block_slots: Vec<(u64, Vec<Vec<u8>>)> = BLOCK_SLOTS
            .range(
                deps.storage,
                None,
                Some(Bound::inclusive(env.block.height)),
                Order::Ascending,
            )
            .collect::<StdResult<_>>()?;

        let block_tasks = block_slots
            .iter()
            .fold(0, |acc, (_, hashes)| acc + hashes.len()) as u64;

        let time_slot: Vec<(u64, Vec<Vec<u8>>)> = TIME_SLOTS
            .range(
                deps.storage,
                None,
                Some(Bound::inclusive(env.block.time.nanos())),
                Order::Ascending,
            )
            .collect::<StdResult<_>>()?;

        let cron_tasks = time_slot
            .iter()
            .fold(0, |acc, (_, hashes)| acc + hashes.len()) as u64;
        Ok(SlotTasksTotalResponse {
            block_tasks,
            cron_tasks,
        })
    }
}

/// Get the slot with lowest height/timestamp
/// NOTE: This prioritizes blocks over timestamps
fn query_get_current_task(deps: Deps, env: Env) -> StdResult<Option<TaskResponse>> {
    let config = CONFIG.load(deps.storage)?;

    let mut block_slot: Vec<(u64, Vec<Vec<u8>>)> = BLOCK_SLOTS
        .range(
            deps.storage,
            None,
            Some(Bound::inclusive(env.block.height)),
            Order::Ascending,
        )
        .take(1)
        .collect::<StdResult<_>>()?;
    if !block_slot.is_empty() {
        let task_hash = block_slot.pop().unwrap().1.pop().unwrap();
        let task = tasks_map().load(deps.storage, &task_hash)?;
        Ok(Some(task.into_response(&config.chain_name)))
    } else {
        let mut time_slot: Vec<(u64, Vec<Vec<u8>>)> = TIME_SLOTS
            .range(
                deps.storage,
                None,
                Some(Bound::inclusive(env.block.time.nanos())),
                Order::Ascending,
            )
            .take(1)
            .collect::<StdResult<_>>()?;
        if !time_slot.is_empty() {
            let task_hash = time_slot.pop().unwrap().1.pop().unwrap();
            let task = tasks_map().load(deps.storage, &task_hash)?;
            Ok(Some(task.into_response(&config.chain_name)))
        } else {
            return Ok(None);
        }
    }
}

fn query_tasks(
    deps: Deps,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> StdResult<Vec<TaskResponse>> {
    let config = CONFIG.load(deps.storage)?;

    let from_index = from_index.unwrap_or_default();
    let limit = limit.unwrap_or(100);

    tasks_map()
        .range(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .map(|task_res| task_res.map(|(_, task)| task.into_response(&config.chain_name)))
        .collect()
}

fn query_tasks_with_queries(
    deps: Deps,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> StdResult<Vec<TaskResponse>> {
    let config = CONFIG.load(deps.storage)?;
    let from_index = from_index.unwrap_or_default();
    let limit = limit.unwrap_or(100);

    tasks_with_queries_map()
        .range(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .map(|task_res| task_res.map(|(_, task)| task.into_response(&config.chain_name)))
        .collect()
}

fn query_tasks_by_owner(
    deps: Deps,
    owner_addr: String,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> StdResult<Vec<TaskResponse>> {
    let owner_addr = deps.api.addr_validate(&owner_addr)?;
    let config = CONFIG.load(deps.storage)?;

    let from_index = from_index.unwrap_or_default();
    let limit = limit.unwrap_or(100);

    let tasks = tasks_map().idx.owner.prefix(owner_addr.clone()).range(
        deps.storage,
        None,
        None,
        Order::Ascending,
    );
    let tasks_with_queries = tasks_with_queries_map().idx.owner.prefix(owner_addr).range(
        deps.storage,
        None,
        None,
        Order::Ascending,
    );
    tasks
        .chain(tasks_with_queries)
        .skip(from_index as usize)
        .take(limit as usize)
        .map(|task_res| task_res.map(|(_, task)| task.into_response(&config.chain_name)))
        .collect()
}

fn query_task(deps: Deps, task_hash: String) -> StdResult<Option<TaskResponse>> {
    let config = CONFIG.load(deps.storage)?;

    if let Some(task) = tasks_map().may_load(deps.storage, task_hash.as_bytes())? {
        Ok(Some(task.into_response(&config.chain_name)))
    } else if let Some(task) =
        tasks_with_queries_map().may_load(deps.storage, task_hash.as_bytes())?
    {
        Ok(Some(task.into_response(&config.chain_name)))
    } else {
        Ok(None)
    }
}

fn query_task_hash(deps: Deps, task: Task) -> StdResult<String> {
    let config = CONFIG.load(deps.storage)?;
    Ok(task.to_hash(&config.chain_name))
}

fn query_slot_hashes(deps: Deps, slot: Option<u64>) -> StdResult<SlotHashesResponse> {
    let mut block_id: u64 = 0;
    let mut block_hashes: Vec<Vec<u8>> = Vec::new();
    let mut time_id: u64 = 0;
    let mut time_hashes: Vec<Vec<u8>> = Vec::new();

    // Check if slot was supplied, otherwise get the next slots for block and time
    if let Some(id) = slot {
        block_hashes = BLOCK_SLOTS.may_load(deps.storage, id)?.unwrap_or_default();
        if !block_hashes.is_empty() {
            block_id = id;
        }
        time_hashes = TIME_SLOTS.may_load(deps.storage, id)?.unwrap_or_default();
        if !time_hashes.is_empty() {
            time_id = id;
        }
    } else {
        let time: Vec<(u64, _)> = TIME_SLOTS
            .range(deps.storage, None, None, Order::Ascending)
            .take(1)
            .collect::<StdResult<Vec<(u64, _)>>>()?;

        if !time.is_empty() {
            // (time_id, time_hashes) = time[0].clone();
            let slot = time[0].clone();
            time_id = slot.0;
            time_hashes = slot.1;
        }

        let block: Vec<(u64, _)> = BLOCK_SLOTS
            .range(deps.storage, None, None, Order::Ascending)
            .take(1)
            .collect::<StdResult<Vec<(u64, _)>>>()?;

        if !block.is_empty() {
            // (block_id, block_hashes) = block[0].clone();
            let slot = block[0].clone();
            block_id = slot.0;
            block_hashes = slot.1;
        }
    }

    // Generate strings for all hashes
    let block_task_hash: Vec<_> = block_hashes
        .iter()
        .map(|b| String::from_utf8(b.to_vec()).unwrap_or_else(|_| "".to_string()))
        .collect();
    let time_task_hash: Vec<_> = time_hashes
        .iter()
        .map(|t| String::from_utf8(t.to_vec()).unwrap_or_else(|_| "".to_string()))
        .collect();

    Ok(SlotHashesResponse {
        block_id,
        block_task_hash,
        time_id,
        time_task_hash,
    })
}

fn query_slot_ids(
    deps: Deps,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> StdResult<SlotIdsResponse> {
    let from_index = from_index.unwrap_or_default();
    let limit = limit.unwrap_or(100);

    let time_ids = TIME_SLOTS
        .keys(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .collect::<StdResult<_>>()?;
    let block_ids = BLOCK_SLOTS
        .keys(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .collect::<StdResult<_>>()?;

    Ok(SlotIdsResponse {
        time_ids,
        block_ids,
    })
}
