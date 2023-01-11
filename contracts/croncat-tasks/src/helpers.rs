use cosmwasm_std::CosmosMsg;
use croncat_sdk_tasks::types::{Boundary, BoundaryValidated, TaskRequest, AmountForOneTask};

use crate::ContractError;

pub(crate) fn validate_boundary(boundary: Option<Boundary>) -> Result<BoundaryValidated, ContractError> {
    match boundary {
        Some(Boundary::Height {
            start: Some(start),
            end: Some(end),
        }) if start > end => Err(ContractError::InvalidBoundary {}),
        Some(Boundary::Height { start, end }) => Ok(BoundaryValidated {
            start: start.map(Into::into),
            end: end.map(Into::into),
            is_block_boundary: true,
        }),
        Some(Boundary::Time { start, end }) => Ok(BoundaryValidated {
            start: start.map(|s| s.nanos()),
            end: end.map(|e| e.nanos()),
            is_block_boundary: false,
        }),
        None => Ok(BoundaryValidated {
            start: None,
            end: None,
            is_block_boundary: true,
        }),
    }
}


pub(crate) fn validate_msg_calculate_usage(api: &dyn Api, task: &TaskRequest, config: &Config) -> Result<(AmountForOneTask, u64),ContractError> {
    let mut gas_amount: u64 = base_gas;
        let mut amount_for_one_task = AmountForOneTask{ native: 0, cw20: None, ibc: None };

        for action in self.actions.iter() {
            // checked for cases, where task creator intentionaly tries to overflow
            gas_amount = gas_amount
                .checked_add(action.gas_limit.unwrap_or(action_gas))
                .ok_or(CoreError::InvalidWasmMsg {})?;
            match &action.msg {
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr,
                    funds: _,
                    msg,
                }) => {
                    // TODO: Is there any way sender can be "self" creating a malicious task?
                    // cannot be THIS contract id, unless predecessor is owner of THIS contract
                    if contract_addr == self_addr && sender != owner_id {
                        return Err(CoreError::InvalidAction {});
                    }
                    if action.gas_limit.is_none() {
                        return Err(CoreError::NoGasLimit {});
                    }
                    if let Ok(cw20_msg) = cosmwasm_std::from_binary(msg) {
                        match cw20_msg {
                            Cw20ExecuteMsg::Send { amount, .. } if !amount.is_zero() => {
                                amount_for_one_task
                                    .cw20
                                    .find_checked_add(&Cw20CoinVerified {
                                        address: api.addr_validate(contract_addr)?,
                                        amount,
                                    })?
                            }
                            Cw20ExecuteMsg::Transfer { amount, .. } if !amount.is_zero() => {
                                amount_for_one_task
                                    .cw20
                                    .find_checked_add(&Cw20CoinVerified {
                                        address: api.addr_validate(contract_addr)?,
                                        amount,
                                    })?
                            }
                            _ => {
                                return Err(CoreError::InvalidAction {});
                            }
                        }
                    }
                }
                CosmosMsg::Staking(StakingMsg::Delegate {
                    validator: _,
                    amount,
                }) => {
                    // Must attach enough balance for staking
                    if amount.amount.is_zero() {
                        return Err(CoreError::InvalidAction {});
                    }
                    amount_for_one_task.native.find_checked_add(amount)?;
                }
                // TODO: Allow send, as long as coverage of assets is correctly handled
                CosmosMsg::Bank(BankMsg::Send {
                    to_address: _,
                    amount,
                }) => {
                    // Restrict bank msg for time being, so contract doesnt get drained, however could allow an escrow type setup
                    // Do something silly to keep it simple. Ensure they only sent one kind of native token and it's testnet Juno
                    // Remember total_deposit is set in tasks.rs when a task is created, and assigned to info.funds
                    // which is however much was passed in, like 1000000ujunox below:
                    // junod tx wasm execute … … --amount 1000000ujunox
                    if amount.iter().any(|coin| coin.amount.is_zero()) {
                        return Err(CoreError::InvalidAction {});
                    }
                    amount_for_one_task.checked_add_native(amount)?;
                }
                CosmosMsg::Bank(_) => {
                    // Restrict bank msg for time being, so contract doesnt get drained, however could allow an escrow type setup
                    return Err(CoreError::InvalidAction {});
                }
                CosmosMsg::Gov(GovMsg::Vote { .. }) => {
                    // Restrict bank msg for time being, so contract doesnt get drained, however could allow an escrow type setup
                    return Err(CoreError::InvalidAction {});
                }
                // TODO: Setup better support for IBC
                CosmosMsg::Ibc(IbcMsg::Transfer { .. }) => {
                    // Restrict bank msg for time being, so contract doesnt get drained, however could allow an escrow type setup
                    return Err(CoreError::InvalidAction {});
                }
                // TODO: Check authZ messages
                _ => (),
            }
        }
        Ok((amount_for_one_task, gas_amount))
    }
