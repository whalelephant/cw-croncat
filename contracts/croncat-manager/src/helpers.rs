use cosmwasm_std::{
    Addr, BankMsg, Coin, CosmosMsg, Empty, MessageInfo, QuerierWrapper, StdError, StdResult,
     Uint128,
};
use croncat_sdk_agents::msg::AgentResponse;
use croncat_sdk_manager::types::Config;

use crate::ContractError;

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
pub(crate) fn query_agent_addr(
    querier: &QuerierWrapper<Empty>,
    config: &Config,
) -> Result<Addr, ContractError> {
    let (tasks_name, version) = &config.croncat_agents_key;
    croncat_factory::state::CONTRACT_ADDRS
        .query(
            querier,
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

pub(crate) fn gas_with_fees(gas_amount: u64, fee: u64) -> Result<u64, ContractError> {
    gas_amount
        .checked_mul(fee)
        .and_then(|n| n.checked_div(100))
        .and_then(|n| n.checked_add(gas_amount))
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
pub(crate) fn assert_caller_is_agent_contract(
    deps_queries: &QuerierWrapper<Empty>,
    config: &Config,
    sender: &Addr,
) -> Result<(), ContractError> {
    let addr = query_agent_addr(deps_queries, config)?;
    if addr != *sender {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn query_agent(
    querier: &QuerierWrapper<Empty>,
    config: &Config,
    agent_id: String,
) -> Result<AgentResponse, ContractError> {
    let addr = query_agent_addr(querier, config)?;
    // Get the agent from the agent contract
    let response: AgentResponse = querier.query_wasm_smart(
        addr,
        &croncat_sdk_agents::msg::QueryMsg::GetAgent {
            account_id: agent_id,
        },
    )?;

    Ok(response)
}

pub(crate) fn create_bank_send_message(
    to: &Addr,
    denom: &str,
    amount: u128,
) -> StdResult<CosmosMsg> {
    let coin = Coin {
        denom: denom.to_owned(),
        amount: Uint128::from(amount),
    };
    let msg = BankMsg::Send {
        to_address: to.into(),
        amount: vec![coin],
    };

    Ok(msg.into())
}
