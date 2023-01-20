use cosmwasm_std::{Addr, Api, BankMsg, BlockInfo, CosmosMsg, WasmMsg};
use croncat_sdk_tasks::types::{
    AmountForOneTask, Boundary, BoundaryValidated, Config, Interval, TaskRequest,
};
use cw20::{Cw20CoinVerified, Cw20ExecuteMsg};

use crate::ContractError;

pub(crate) fn validate_boundary(
    block_info: &BlockInfo,
    boundary: &Option<Boundary>,
    interval: &Interval,
) -> Result<BoundaryValidated, ContractError> {
    let prevalid_boundary = match (interval, boundary) {
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

    if let Some(end) = prevalid_boundary.end {
        if end >= prevalid_boundary.start {
            return Err(ContractError::InvalidBoundary {});
        }
    }
    Ok(prevalid_boundary)
}

pub(crate) fn validate_msg_calculate_usage(
    api: &dyn Api,
    task: &TaskRequest,
    self_addr: &Addr,
    sender: &Addr,
    config: &Config,
) -> Result<AmountForOneTask, ContractError> {
    let mut amount_for_one_task = AmountForOneTask {
        gas: config.gas_base_fee,
        cw20: None,
        coin: None,
    };

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
                // TODO: Is there any way sender can be "self" creating a malicious task?
                // cannot be THIS contract id, unless predecessor is owner of THIS contract
                // TODO(buckram): probably should make it check manager address as well
                if contract_addr == self_addr && *sender != config.owner_addr {
                    return Err(ContractError::InvalidAction {});
                }
                if action.gas_limit.is_none() {
                    return Err(ContractError::NoGasLimit {});
                }
                if let Ok(cw20_msg) = cosmwasm_std::from_binary(msg) {
                    match cw20_msg {
                        Cw20ExecuteMsg::Send { amount, .. } if !amount.is_zero() => {
                            if !amount_for_one_task.add_cw20(Cw20CoinVerified {
                                address: api.addr_validate(contract_addr)?,
                                amount,
                            }) {
                                return Err(ContractError::InvalidAction {});
                            }
                        }
                        Cw20ExecuteMsg::Transfer { amount, .. } if !amount.is_zero() => {
                            if !amount_for_one_task.add_cw20(Cw20CoinVerified {
                                address: api.addr_validate(contract_addr)?,
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
                if amount.len() != 1
                    || amount[0].amount.is_zero()
                    || amount_for_one_task.add_coin(amount[0].clone())
                {
                    return Err(ContractError::InvalidAction {});
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
