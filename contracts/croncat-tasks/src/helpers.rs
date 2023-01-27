use cosmwasm_std::{
    Addr, BankMsg, Binary, BlockInfo, CosmosMsg, Deps, Empty, Order, QuerierWrapper, StdResult,
    Storage, WasmMsg,
};
use croncat_sdk_tasks::types::{
    AmountForOneTask, Boundary, BoundaryValidated, Config, Interval, TaskRequest,
};
use cw20::{Cw20CoinVerified, Cw20ExecuteMsg};

use crate::{
    state::{
        tasks_map, tasks_with_queries_map, BLOCK_MAP_QUERIES, BLOCK_SLOTS, TASKS_TOTAL,
        TASKS_WITH_QUERIES_TOTAL, TIME_MAP_QUERIES, TIME_SLOTS,
    },
    ContractError,
};

pub(crate) fn validate_boundary(
    block_info: &BlockInfo,
    boundary: &Option<Boundary>,
    interval: &Interval,
) -> Result<BoundaryValidated, ContractError> {
    let boundary_validated = match (interval, boundary) {
        (Interval::Cron(_), Some(Boundary::Height { .. }))
        | (Interval::Block(_), Some(Boundary::Time { .. })) => {
            Err(ContractError::InvalidBoundary {})
        }
        (_, Some(Boundary::Height { start, end })) => Ok(BoundaryValidated {
            start: start.map(Into::into).unwrap_or(block_info.height),
            end: end.map(Into::into),
            is_block_boundary: true,
        }),
        (_, Some(Boundary::Time { start, end })) => Ok(BoundaryValidated {
            start: start.unwrap_or(block_info.time).nanos(),
            end: end.map(|e| e.nanos()),
            is_block_boundary: false,
        }),
        (Interval::Cron(_), None) => Ok(BoundaryValidated {
            start: block_info.time.nanos(),
            end: None,
            is_block_boundary: false,
        }),
        // Defaults to block boundary rest
        (_, None) => Ok(BoundaryValidated {
            start: block_info.height,
            end: None,
            is_block_boundary: true,
        }),
    }?;

    if let Some(end) = boundary_validated.end {
        if boundary_validated.start >= end {
            return Err(ContractError::InvalidBoundary {});
        }
    }
    Ok(boundary_validated)
}

/// Check for calls of our contracts
pub(crate) fn check_for_self_calls(
    tasks_addr: &Addr,
    manager_addr: &Addr,
    agents_addr: &Addr,
    manager_owner_addr: &Addr,
    sender: &Addr,
    contract_addr: &String,
    msg: &Binary,
) -> Result<(), ContractError> {
    // If it one of the our contracts it should be a manager
    if contract_addr == tasks_addr || contract_addr == agents_addr {
        return Err(ContractError::InvalidAction {});
    } else if contract_addr == manager_addr {
        // Check if caller is manager owner
        if sender != manager_owner_addr {
            return Err(ContractError::InvalidAction {});
        } else if let Ok(msg) = cosmwasm_std::from_binary(msg) {
            // Check if it's tick
            if !matches!(msg, croncat_sdk_manager::msg::ManagerExecuteMsg::Tick {}) {
                return Err(ContractError::InvalidAction {});
            }
            // Other messages not allowed
        } else {
            return Err(ContractError::InvalidAction {});
        }
    }
    Ok(())
}

pub(crate) fn validate_msg_calculate_usage(
    deps: Deps,
    task: &TaskRequest,
    self_addr: &Addr,
    sender: &Addr,
    config: &Config,
) -> Result<AmountForOneTask, ContractError> {
    let mut amount_for_one_task = AmountForOneTask {
        gas: config.gas_base_fee,
        cw20: None,
        coin: [None, None],
    };

    let manager_addr = get_manager_addr(&deps.querier, config)?;
    let agents_addr = get_agents_addr(&deps.querier, config)?;

    let manager_conf: croncat_sdk_manager::types::Config = deps.querier.query_wasm_smart(
        &manager_addr,
        &croncat_sdk_manager::msg::ManagerQueryMsg::Config {},
    )?;

    for action in task.actions.iter() {
        if !amount_for_one_task.add_gas(
            action.gas_limit.unwrap_or(config.gas_action_fee),
            config.gas_limit,
        ) {
            return Err(ContractError::InvalidAction {});
        }
        match &action.msg {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr,
                funds: _,
                msg,
            }) => {
                if action.gas_limit.is_none() {
                    return Err(ContractError::NoGasLimit {});
                }
                check_for_self_calls(
                    self_addr,
                    &manager_addr,
                    &agents_addr,
                    &manager_conf.owner_addr,
                    sender,
                    contract_addr,
                    msg,
                )?;
                if let Ok(cw20_msg) = cosmwasm_std::from_binary(msg) {
                    match cw20_msg {
                        Cw20ExecuteMsg::Send { amount, .. } if !amount.is_zero() => {
                            if !amount_for_one_task.add_cw20(Cw20CoinVerified {
                                address: deps.api.addr_validate(contract_addr)?,
                                amount,
                            }) {
                                return Err(ContractError::InvalidAction {});
                            }
                        }
                        Cw20ExecuteMsg::Transfer { amount, .. } if !amount.is_zero() => {
                            if !amount_for_one_task.add_cw20(Cw20CoinVerified {
                                address: deps.api.addr_validate(contract_addr)?,
                                amount,
                            }) {
                                return Err(ContractError::InvalidAction {});
                            }
                        }
                        _ => {
                            return Err(ContractError::InvalidAction {});
                        }
                    }
                }
            }
            CosmosMsg::Bank(BankMsg::Send {
                to_address: _,
                amount,
            }) => {
                // Restrict bank msg for time being, so contract doesnt get drained, however could allow an escrow type setup
                // Do something silly to keep it simple. Ensure they only sent one kind of native token and it's testnet Juno
                // Remember total_deposit is set in tasks.rs when a task is created, and assigned to info.funds
                // which is however much was passed in, like 1000000ujunox below:
                // junod tx wasm execute … … --amount 1000000ujunox
                if amount.len() > 2 {
                    return Err(ContractError::InvalidAction {});
                }
                for coin in amount {
                    if coin.amount.is_zero() || !amount_for_one_task.add_coin(coin.clone())? {
                        return Err(ContractError::InvalidAction {});
                    }
                }
            }
            // Disallow unknown messages
            _ => {
                return Err(ContractError::InvalidAction {});
            }
        }
    }
    Ok(amount_for_one_task)
}

pub(crate) fn remove_task_without_queries(
    storage: &mut dyn Storage,
    hash: &[u8],
    is_block: bool,
) -> StdResult<()> {
    tasks_map().remove(storage, hash)?;
    TASKS_TOTAL.update(storage, |total| StdResult::Ok(total - 1))?;
    if is_block {
        let blocks = BLOCK_SLOTS
            .range(storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;
        for (bid, mut block_hashes) in blocks {
            let found = false;
            block_hashes.retain(|h| {
                let found = h == hash;
                !found
            });
            if found {
                if block_hashes.is_empty() {
                    BLOCK_SLOTS.remove(storage, bid);
                } else {
                    BLOCK_SLOTS.save(storage, bid, &block_hashes)?;
                }
                break;
            }
        }
    } else {
        let time_buckets = TIME_SLOTS
            .range(storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;
        for (tid, mut time_hashes) in time_buckets {
            let found = false;
            time_hashes.retain(|h| {
                let found = h == hash;
                !found
            });
            if found {
                if time_hashes.is_empty() {
                    TIME_SLOTS.remove(storage, tid);
                } else {
                    TIME_SLOTS.save(storage, tid, &time_hashes)?;
                }
                break;
            }
        }
    }

    Ok(())
}

pub(crate) fn remove_task_with_queries(
    storage: &mut dyn Storage,
    hash: &[u8],
    is_block: bool,
) -> StdResult<()> {
    tasks_with_queries_map().remove(storage, hash)?;
    if is_block {
        BLOCK_MAP_QUERIES.remove(storage, hash)
    } else {
        TIME_MAP_QUERIES.remove(storage, hash)
    }
    TASKS_WITH_QUERIES_TOTAL.update(storage, |total| StdResult::Ok(total - 1))?;
    Ok(())
}

pub(crate) fn check_if_sender_is_manager(
    deps_queries: &QuerierWrapper<Empty>,
    config: &Config,
    sender: &Addr,
) -> Result<(), ContractError> {
    let manager_addr = get_manager_addr(deps_queries, config)?;
    if manager_addr != *sender {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

pub(crate) fn get_manager_addr(
    deps_queries: &QuerierWrapper<Empty>,
    config: &Config,
) -> Result<Addr, ContractError> {
    let (manager_name, version) = &config.croncat_manager_key;
    croncat_factory::state::CONTRACT_ADDRS
        .query(
            deps_queries,
            config.croncat_factory_addr.clone(),
            (&manager_name, version),
        )?
        .ok_or(ContractError::InvalidKey {})
}

pub(crate) fn get_agents_addr(
    deps_queries: &QuerierWrapper<Empty>,
    config: &Config,
) -> Result<Addr, ContractError> {
    let (agents_name, version) = &config.croncat_agents_key;
    croncat_factory::state::CONTRACT_ADDRS
        .query(
            deps_queries,
            config.croncat_factory_addr.clone(),
            (&agents_name, version),
        )?
        .ok_or(ContractError::InvalidKey {})
}
