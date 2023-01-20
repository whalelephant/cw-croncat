#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use croncat_sdk_tasks::types::{Config, SlotType, Task, TaskRequest};
use cw2::{query_contract_info, set_contract_version};
use cw20::Cw20CoinVerified;

use crate::error::ContractError;
use crate::helpers::{validate_boundary, validate_msg_calculate_usage};
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
        croncat_factory_addr,
        owner_addr,
        croncat_manager_key,
        croncat_agents_key,
        slot_granularity_time,
        gas_base_fee,
        gas_action_fee,
        gas_query_fee,
        gas_limit,
    } = msg;
    let config = Config {
        paused: false,
        owner_addr: owner_addr
            .map(|human| deps.api.addr_validate(&human))
            .transpose()?
            .unwrap_or(info.sender),
        croncat_factory_addr: deps.api.addr_validate(&croncat_factory_addr)?,
        croncat_manager_key,
        croncat_agents_key,
        slot_granularity_time: slot_granularity_time.unwrap_or(SLOT_GRANULARITY_TIME),
        gas_base_fee: gas_base_fee.unwrap_or(GAS_BASE_FEE),
        gas_action_fee: gas_action_fee.unwrap_or(GAS_ACTION_FEE),
        gas_query_fee: gas_query_fee.unwrap_or(GAS_QUERY_FEE),
        gas_limit: gas_limit.unwrap_or(GAS_LIMIT),
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // TODO: check config if paused
    match msg {
        ExecuteMsg::CreateTask { task } => execute_create_task(deps, env, info, task),
        ExecuteMsg::RemoveTask { task_hash } => todo!(),
    }
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

    // Validate cw20
    let verified_cw20 = task
        .cw20
        .map(|cw20| -> StdResult<_> {
            Ok(Cw20CoinVerified {
                address: deps.api.addr_validate(&cw20.address)?,
                amount: cw20.amount,
            })
        })
        .transpose()?;

    let version = query_contract_info(&deps.querier, env.contract.address.as_str())?.version;
    let item = Task {
        owner_addr,
        interval: task.interval,
        boundary,
        stop_on_fail: task.stop_on_fail,
        amount_for_one_task,
        actions: task.actions,
        queries: task.queries.unwrap_or_default(),
        transforms: task.transforms.unwrap_or_default(),
        version,
    };
    let hash_prefix = &env.block.chain_id;
    let hash = item.to_hash(hash_prefix);

    let (next_id, slot_kind) =
        item.interval
            .next(&env, &item.boundary, config.slot_granularity_time);
    if next_id == 0 {
        return Err(ContractError::TaskEnded {});
    }

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

    // TODO: pass message to manager with amount_for_one_task
    // TODO: ping agent to notify new task is arrived
    Ok(Response::new()
        .set_data(hash.as_bytes())
        .add_attribute("action", "create_task")
        .add_attribute("slot_id", next_id.to_string())
        .add_attribute("slot_kind", slot_kind.to_string())
        .add_attribute("task_hash", hash)
        .add_attribute("with_queries", with_queries.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    todo!();
}
