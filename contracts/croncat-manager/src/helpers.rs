use cosmwasm_std::{
    coin, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Empty, MessageInfo, QuerierWrapper, Reply,
    Response, StdError, StdResult, Storage, SubMsg, Uint128, WasmMsg,
};
use croncat_sdk_core::types::AmountForOneTask;
use croncat_sdk_manager::types::{Config, TaskBalance};
use cw20::{Cw20CoinVerified, Cw20ExecuteMsg};

use crate::{
    balances::{add_fee_rewards, add_user_cw20},
    contract::TASK_REPLY,
    state::{QueueItem, CONFIG, REPLY_QUEUE, TASKS_BALANCES},
    ContractError,
};

/// Check if contract is paused or user attached redundant funds.
/// Called before every method, except [crate::contract::execute_update_config]
pub(crate) fn check_ready_for_execution(
    info: &MessageInfo,
    config: &Config,
) -> Result<(), ContractError> {
    if config.paused {
        Err(ContractError::Paused {})
    } else if !info.funds.is_empty() {
        Err(ContractError::RedundantFunds {})
    } else {
        Ok(())
    }
}

pub(crate) fn get_tasks_addr(
    deps_queries: &QuerierWrapper<Empty>,
    config: &Config,
) -> Result<Addr, ContractError> {
    let (tasks_name, version) = &config.croncat_tasks_key;
    croncat_factory::state::CONTRACT_ADDRS
        .query(
            deps_queries,
            config.croncat_factory_addr.clone(),
            (tasks_name, version),
        )?
        .ok_or(ContractError::InvalidKey {})
}

pub(crate) fn check_if_sender_is_tasks(
    deps_queries: &QuerierWrapper<Empty>,
    config: &Config,
    sender: &Addr,
) -> Result<(), ContractError> {
    let tasks_addr = get_tasks_addr(deps_queries, config)?;
    if tasks_addr != *sender {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
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
            (agents_name, version),
        )?
        .ok_or(ContractError::InvalidKey {})
}

pub(crate) fn gas_with_fees(gas_amount: u64, fee: u64) -> Result<u64, ContractError> {
    gas_fee(gas_amount, fee)?
        .checked_add(gas_amount)
        .ok_or(ContractError::InvalidGasCalculation {})
}

pub(crate) fn gas_fee(gas_amount: u64, fee: u64) -> Result<u64, ContractError> {
    gas_amount
        .checked_mul(fee)
        .and_then(|n| n.checked_div(100))
        .ok_or(ContractError::InvalidGasCalculation {})
}

pub(crate) fn attached_natives(
    native_denom: &str,
    funds: Vec<Coin>,
) -> Result<(Uint128, Option<Coin>), ContractError> {
    let mut ibc: Option<Coin> = None;
    let mut native = Uint128::zero();
    for f in funds {
        if f.denom == native_denom {
            native += f.amount;
        } else if let Some(ibc) = &mut ibc {
            if f.denom == ibc.denom {
                ibc.amount += f.amount
            } else {
                return Err(ContractError::TooManyCoins {});
            }
        } else {
            ibc = Some(f);
        }
    }
    Ok((native, ibc))
}

pub(crate) fn calculate_required_natives(
    amount_for_one_task_coins: [Option<Coin>; 2],
    native_denom: &str,
) -> Result<(Uint128, Option<Coin>), ContractError> {
    let res = match amount_for_one_task_coins {
        [Some(c1), Some(c2)] => {
            if c1.denom == native_denom {
                (c1.amount, Some(c2))
            } else if c2.denom == native_denom {
                (c2.amount, Some(c1))
            } else {
                return Err(StdError::generic_err("none of the coins are native").into());
            }
        }
        [Some(c1), None] => {
            if c1.denom == native_denom {
                (c1.amount, None)
            } else {
                (Uint128::zero(), Some(c1))
            }
        }
        [None, None] => (Uint128::zero(), None),
        [None, Some(_)] => unreachable!(),
    };
    Ok(res)
}

/// Get sub messages for this task
/// To minimize gas consumption for loads we only reply on failure
/// And the last item to calculate rewards and reschedule or removal of the task
pub(crate) fn task_sub_msgs(task: &croncat_sdk_tasks::types::TaskResponse) -> Vec<SubMsg> {
    let mut sub_msgs = Vec::with_capacity(task.actions.len());
    let mut actions_iter = task.actions.iter().enumerate();

    // safe unwrap here, we don't allow empty actions
    let (last_idx, last_action) = actions_iter.next_back().unwrap();

    for (idx, action) in actions_iter {
        if let Some(gas_limit) = action.gas_limit {
            sub_msgs.push(
                SubMsg::reply_on_error(action.msg.clone(), idx as u64).with_gas_limit(gas_limit),
            );
        } else {
            sub_msgs.push(SubMsg::reply_on_error(action.msg.clone(), idx as u64));
        }
    }
    if let Some(gas_limit) = last_action.gas_limit {
        sub_msgs.push(
            SubMsg::reply_always(last_action.msg.clone(), last_idx as u64)
                .with_gas_limit(gas_limit),
        );
    } else {
        sub_msgs.push(SubMsg::reply_always(
            last_action.msg.clone(),
            last_idx as u64,
        ));
    }
    sub_msgs
}

pub(crate) fn parse_reply_msg(
    storage: &mut dyn Storage,
    queue_item: &mut QueueItem,
    msg: Reply,
) -> bool {
    let id = msg.id as usize;
    if let cosmwasm_std::SubMsgResult::Err(err) = msg.result {
        queue_item.failures.push((id as u8, err));
    }
    let last = queue_item.task.actions.len() == id + 1;
    // If last action let's clean state here
    if last {
        REPLY_QUEUE.remove(storage)
    }
    last
}

pub(crate) fn finalize_task(
    deps: DepsMut,
    queue_item: QueueItem,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut task_balance =
        TASKS_BALANCES.load(deps.storage, queue_item.task.task_hash.as_bytes())?;
    // Sub native for gas
    let gas_with_fees = gas_with_fees(
        queue_item.task.amount_for_one_task.gas,
        config.agent_fee + config.treasury_fee,
    )?;
    let native_for_gas_required = config.gas_price.calculate(gas_with_fees)?;
    task_balance.native_balance = task_balance
        .native_balance
        .checked_sub(Uint128::new(native_for_gas_required))
        .map_err(StdError::overflow)?;

    add_fee_rewards(
        deps.storage,
        queue_item.task.amount_for_one_task.gas,
        &config.gas_price,
        &queue_item.agent_addr,
        config.agent_fee,
        config.treasury_fee,
    )?;

    let original_amounts = queue_item.task.amount_for_one_task.clone();
    let amounts_without_failed_txs = amounts_without_failed_txs(&queue_item)?;

    // Sub transferred coins
    for coin in amounts_without_failed_txs.coin.iter().flatten() {
        task_balance.sub_coin(coin, &config.native_denom)?;
    }
    // Sub transferred cw20s
    if let Some(cw20) = &amounts_without_failed_txs.cw20 {
        task_balance.sub_cw20(cw20)?;
    }
    let (native_for_sends_required, ibc_required) =
        calculate_required_natives(original_amounts.coin, &config.native_denom)?;

    // unregister task and return unused deposits if any of this:
    // - not recurring
    // - should stop on fail
    // - task balance drained
    if matches!(
        queue_item.task.interval,
        croncat_sdk_tasks::types::Interval::Once
    ) || (queue_item.task.stop_on_fail && !queue_item.failures.is_empty())
        || task_balance
            .verify_enough_attached(
                native_for_sends_required + Uint128::new(native_for_gas_required),
                original_amounts.cw20,
                ibc_required,
                false,
                &config.native_denom,
            )
            .is_err()
    {
        // Transfer unused balances to the task creator and cw20s to the temp balances
        let coins_transfer = remove_task_balance(
            deps.storage,
            task_balance,
            &queue_item.task.owner_addr,
            &config.native_denom,
            queue_item.task.task_hash.as_bytes(),
        )?;
        // Remove task on tasks contract
        let tasks_addr = get_tasks_addr(&deps.querier, &config)?;
        let msg = croncat_sdk_core::internal_messages::tasks::TasksRemoveTaskByManager {
            task_hash: queue_item.task.task_hash.into_bytes(),
        }
        .into_cosmos_msg(tasks_addr)?;
        let res = Response::new().add_message(msg);
        // Zero transfer will fail tx
        if !coins_transfer.is_empty() {
            Ok(res.add_message(BankMsg::Send {
                to_address: queue_item.task.owner_addr.into_string(),
                amount: coins_transfer,
            }))
        } else {
            Ok(res)
        }
    } else {
        let tasks_addr = get_tasks_addr(&deps.querier, &config)?;
        TASKS_BALANCES.save(
            deps.storage,
            queue_item.task.task_hash.as_bytes(),
            &task_balance,
        )?;
        let msg = croncat_sdk_core::internal_messages::tasks::TasksRescheduleTask {
            task_hash: queue_item.task.task_hash.into_bytes(),
        }
        .into_cosmos_msg(tasks_addr)?;
        Ok(Response::new().add_submessage(SubMsg::reply_always(msg, TASK_REPLY)))
    }
}

pub(crate) fn amounts_without_failed_txs(queue_item: &QueueItem) -> StdResult<AmountForOneTask> {
    let mut amounts = queue_item.task.amount_for_one_task.clone();
    for (idx, _) in queue_item.failures.iter() {
        match &queue_item.task.actions[(*idx) as usize].msg {
            CosmosMsg::Bank(BankMsg::Send { amount, .. }) => {
                for coin in amount {
                    amounts.sub_coin(coin)?;
                }
            }
            CosmosMsg::Wasm(WasmMsg::Execute {
                msg, contract_addr, ..
            }) => {
                if let Ok(cw20_msg) = cosmwasm_std::from_binary(msg) {
                    match cw20_msg {
                        Cw20ExecuteMsg::Send { amount, .. } => {
                            amounts.sub_cw20(&Cw20CoinVerified {
                                // Addr safe here because we checked it at `is_valid_msg_calculate_usage`
                                address: Addr::unchecked(contract_addr),
                                amount,
                            })?;
                        }
                        Cw20ExecuteMsg::Transfer { amount, .. } => {
                            amounts.sub_cw20(&Cw20CoinVerified {
                                address: Addr::unchecked(contract_addr),
                                amount,
                            })?;
                        }
                        _ => (),
                    };
                }
            }
            _ => (),
        }
    }
    Ok(amounts)
}

/// This function will
/// - Consume `TaskBalance`
/// - Move unused cw20's to the temp balances
/// - Return any unused coins for the use in the message
pub(crate) fn remove_task_balance(
    storage: &mut dyn Storage,
    task_balance: TaskBalance,
    task_owner: &Addr,
    native_denom: &str,
    task_hash: &[u8],
) -> StdResult<Vec<Coin>> {
    let mut coins_transfer = vec![];
    if task_balance.native_balance > Uint128::zero() {
        coins_transfer.push(coin(task_balance.native_balance.u128(), native_denom))
    }

    if let Some(ibc) = task_balance.ibc_balance {
        if ibc.amount > Uint128::zero() {
            coins_transfer.push(ibc);
        }
    }

    if let Some(cw20) = task_balance.cw20_balance {
        // Back to the temp balance
        add_user_cw20(storage, task_owner, &cw20)?;
    }
    TASKS_BALANCES.remove(storage, task_hash);
    Ok(coins_transfer)
}
