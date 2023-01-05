#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, WasmMsg,
};
use croncat_sdk_core::balancer;
use croncat_sdk_core::types::{BalancesResponse, UpdateConfig};
use cw2::set_contract_version;
use cw20::{Cw20Coin, Cw20CoinVerified, Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::error::ContractError;
use crate::helpers::{
    add_available_cw20, add_available_native, add_user_cw20, check_ready_for_execution,
    sub_available_cw20, sub_user_cw20,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    Config, AVAILABLE_CW20_BALANCE, AVAILABLE_NATIVE_BALANCE, CONFIG, USERS_BALANCES_CW20,
};

pub(crate) const CONTRACT_NAME: &str = "crates.io:croncat-manager";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_NOMINATION_DURATION: u16 = 360;
/// Value based on non-wasm operations, wasm ops seem impossible to predict
const GAS_BASE_FEE: u64 = 300_000;
/// Gas needed for single action
const GAS_ACTION_FEE: u64 = 130_000;
/// Gas needed for single non-wasm query
const GAS_QUERY_FEE: u64 = 5_000;
/// Gas needed for single wasm query
const GAS_WASM_QUERY_FEE: u64 = 60_000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Deconstruct so we don't miss anything
    let InstantiateMsg {
        denom,
        cw_rules_addr,
        croncat_tasks_addr,
        croncat_agents_addr,
        owner_id,
        gas_base_fee,
        gas_action_fee,
        gas_query_fee,
        gas_wasm_query_fee,
        gas_price,
        agent_nomination_duration,
    } = msg;

    let gas_price = gas_price.unwrap_or_default();
    if !gas_price.is_valid() {
        return Err(ContractError::InvalidGasPrice {});
    }

    let owner_id = owner_id
        .map(|human| deps.api.addr_validate(&human))
        .transpose()?
        .unwrap_or(info.sender);

    let config = Config {
        paused: false,
        owner_id,
        min_tasks_per_agent: 3,
        agents_eject_threshold: 600,
        agent_nomination_duration: agent_nomination_duration.unwrap_or(DEFAULT_NOMINATION_DURATION),
        cw_rules_addr: deps.api.addr_validate(&cw_rules_addr)?,
        croncat_tasks_addr: deps.api.addr_validate(&croncat_tasks_addr)?,
        croncat_agents_addr: deps.api.addr_validate(&croncat_agents_addr)?,
        agent_fee: 5,
        gas_price,
        gas_base_fee: gas_base_fee.map(Into::into).unwrap_or(GAS_BASE_FEE),
        gas_action_fee: gas_action_fee.map(Into::into).unwrap_or(GAS_ACTION_FEE),
        gas_query_fee: gas_query_fee.map(Into::into).unwrap_or(GAS_QUERY_FEE),
        gas_wasm_query_fee: gas_wasm_query_fee
            .map(Into::into)
            .unwrap_or(GAS_WASM_QUERY_FEE),
        slot_granularity_time: 10_000_000_000, // 10 seconds
        cw20_whitelist: vec![],
        native_denom: denom,
        balancer: Default::default(),
        limit: 100,
    };

    CONFIG.save(deps.storage, &config)?;
    for coin in info.funds {
        add_available_native(deps.storage, &coin)?;
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        // TODO?:.add_attribute("config", format!("{:?}, &config"))
        .add_attribute("method", "instantiate")
        .add_attribute("paused", config.paused.to_string())
        .add_attribute("owner_id", config.owner_id.to_string())
        .add_attribute(
            "min_tasks_per_agent",
            config.min_tasks_per_agent.to_string(),
        )
        .add_attribute(
            "agents_eject_threshold",
            config.agents_eject_threshold.to_string(),
        )
        .add_attribute(
            "agent_nomination_duration",
            config.agent_nomination_duration.to_string(),
        )
        .add_attribute("cw_rules_addr", config.cw_rules_addr.to_string())
        .add_attribute("croncat_tasks_addr", config.croncat_tasks_addr.to_string())
        .add_attribute(
            "croncat_agents_addr",
            config.croncat_agents_addr.to_string(),
        )
        .add_attribute("agent_fee", config.agent_fee.to_string())
        .add_attribute("gas_price", format!("{:?}", config.gas_price))
        .add_attribute("gas_base_fee", config.gas_base_fee.to_string())
        .add_attribute("gas_action_fee", config.gas_action_fee.to_string())
        .add_attribute("gas_query_fee", config.gas_query_fee.to_string())
        .add_attribute("gas_wasm_query_fee", config.gas_wasm_query_fee.to_string())
        .add_attribute(
            "slot_granularity_time",
            config.slot_granularity_time.to_string(),
        )
        .add_attribute("cw20_whitelist", format!("{:?}", config.cw20_whitelist))
        .add_attribute("native_denom", config.native_denom.to_string())
        .add_attribute("balancer", format!("{:?}", config.balancer))
        .add_attribute("limit", config.limit.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig(msg) => execute_update_config(deps, info, msg),
        ExecuteMsg::ProxyCall { task_hash: None } => execute_proxy_call(deps, env, info),
        ExecuteMsg::ProxyCall {
            task_hash: Some(task_hash),
        } => execute_proxy_call_with_queries(deps, env, info, task_hash),
        ExecuteMsg::Receive(msg) => execute_receive_cw20(deps, info, msg),
        ExecuteMsg::WithdrawWalletBalances { cw20_amounts } => {
            execute_withdraw_wallet_balances(deps, info, cw20_amounts)
        }
        ExecuteMsg::Tick {} => execute_tick(deps, env, info),
    }
}

fn execute_proxy_call(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    check_ready_for_execution(&info, &config)?;

    // TODO: query agent to check if ready
    // TODO: execute task

    Ok(Response::new().add_attribute("action", "proxy_call"))
}

fn execute_proxy_call_with_queries(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    task_hash: String,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    check_ready_for_execution(&info, &config)?;

    // TODO: query agent to check if ready
    // TODO: execute task

    Ok(Response::new().add_attribute("action", "proxy_call_with_queries"))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    msg: UpdateConfig,
) -> Result<Response, ContractError> {
    let new_config = CONFIG.update(deps.storage, |config| {
        if info.sender != config.owner_id {
            return Err(ContractError::Unauthorized {});
        }

        let gas_price = msg.gas_price.unwrap_or(config.gas_price);
        if gas_price.is_valid() {
            return Err(ContractError::InvalidGasPrice {});
        }

        let owner_id = msg
            .owner_id
            .map(|human| deps.api.addr_validate(&human))
            .transpose()?
            .unwrap_or(config.owner_id);

        let new_config = Config {
            paused: msg.paused.unwrap_or(config.paused),
            owner_id,
            min_tasks_per_agent: msg
                .min_tasks_per_agent
                .unwrap_or(config.min_tasks_per_agent),
            agents_eject_threshold: msg
                .agents_eject_threshold
                .unwrap_or(config.agents_eject_threshold),
            agent_nomination_duration: config.agent_nomination_duration,
            cw_rules_addr: config.cw_rules_addr,
            croncat_tasks_addr: config.croncat_tasks_addr,
            croncat_agents_addr: config.croncat_agents_addr,
            agent_fee: msg.agent_fee.unwrap_or(config.agent_fee),
            gas_price,
            gas_base_fee: msg.gas_base_fee.unwrap_or(config.gas_base_fee),
            gas_action_fee: msg.gas_action_fee.unwrap_or(config.gas_action_fee),
            gas_query_fee: msg.gas_query_fee.unwrap_or(config.gas_query_fee),
            gas_wasm_query_fee: msg.gas_wasm_query_fee.unwrap_or(config.gas_wasm_query_fee),
            slot_granularity_time: msg
                .slot_granularity_time
                .unwrap_or(config.slot_granularity_time),
            cw20_whitelist: config.cw20_whitelist,
            native_denom: config.native_denom,
            balancer: msg.balancer.unwrap_or(config.balancer),
            limit: config.limit,
        };
        Ok(new_config)
    })?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("config", format!("{new_config:?}")))
}

pub fn execute_receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    check_ready_for_execution(&info, &config)?;

    let sender = deps.api.addr_validate(&msg.sender)?;
    let coin_addr = info.sender;

    let cw20_verified = Cw20CoinVerified {
        address: coin_addr,
        amount: msg.amount,
    };
    let user_cw20_balance = add_user_cw20(deps.storage, &sender, &cw20_verified)?;
    let available_cw20_balance = add_available_cw20(deps.storage, &cw20_verified)?;
    Ok(Response::new()
        .add_attribute("action", "receive_cw20")
        .add_attribute("cw20_received", cw20_verified.to_string())
        .add_attribute("available_cw20_balance", available_cw20_balance)
        .add_attribute("user_cw20_balance", user_cw20_balance))
}

pub fn execute_withdraw_wallet_balances(
    deps: DepsMut,
    info: MessageInfo,
    cw20_amounts: Vec<Cw20Coin>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    check_ready_for_execution(&info, &config)?;

    let wallet = info.sender;
    let withdraws: Vec<Cw20CoinVerified> = cw20_amounts
        .into_iter()
        .map(|cw20| {
            let address = deps.api.addr_validate(&cw20.address)?;
            Ok(Cw20CoinVerified {
                address,
                amount: cw20.amount,
            })
        })
        .collect::<StdResult<_>>()?;

    // update user and croncat manager balances
    let mut updated_user_cw20 = Vec::with_capacity(withdraws.len());
    let mut updated_available_cw20 = Vec::with_capacity(withdraws.len());
    for cw20 in withdraws.iter() {
        let new_user_bal = sub_user_cw20(deps.storage, &wallet, cw20)?;
        let new_avail_bal = sub_available_cw20(deps.storage, cw20)?;
        updated_user_cw20.push(Cw20CoinVerified {
            address: cw20.address.clone(),
            amount: new_user_bal,
        });
        updated_available_cw20.push(Cw20CoinVerified {
            address: cw20.address.clone(),
            amount: new_avail_bal,
        })
    }

    let msgs = {
        let mut msgs = Vec::with_capacity(withdraws.len());
        for wd in withdraws {
            msgs.push(WasmMsg::Execute {
                contract_addr: wd.address.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: wallet.to_string(),
                    amount: wd.amount,
                })?,
                funds: vec![],
            });
        }
        msgs
    };

    Ok(Response::new()
        .add_attribute("action", "withdraw_wallet_balances")
        .add_attribute(
            "updated_user_cw20_balances",
            format!("{updated_user_cw20:?}"),
        )
        .add_attribute(
            "updated_available_cw20_balances",
            format!("{updated_available_cw20:?}"),
        )
        .add_messages(msgs))
}

/// Helps manage and cleanup agents
/// Deletes agents which missed more than agents_eject_threshold slot
// TODO: It might be not possible to deserialize all of the active agents, need to find better solution
// See issue #247
pub fn execute_tick(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    check_ready_for_execution(&info, &config)?;

    // let current_slot = env.block.height;
    // let cfg = CONFIG.load(deps.storage)?;
    // let mut attributes = vec![];
    // let mut submessages = vec![];

    // for agent_id in self.agent_active_queue.load(deps.storage)? {
    //     let agent = self.agents.load(deps.storage, &agent_id)?;
    //     if current_slot > agent.last_executed_slot + cfg.agents_eject_threshold {
    //         let resp = self
    //             .unregister_agent(deps.storage, &agent_id, None)
    //             .unwrap_or_default();
    //         // Save attributes and messages
    //         attributes.extend_from_slice(&resp.attributes);
    //         submessages.extend_from_slice(&resp.messages);
    //     }
    // }

    // // Check if there isn't any active or pending agents
    // if self.agent_active_queue.load(deps.storage)?.is_empty()
    //     && self.agent_pending_queue.is_empty(deps.storage)?
    // {
    //     attributes.push(Attribute::new("lifecycle", "tick_failure"))
    // }
    Ok(Response::new().add_attribute("action", "tick"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::Balances { from_index, limit } => {
            to_binary(&query_balances(deps, from_index, limit)?)
        }
        QueryMsg::Cw20WalletBalances {
            wallet,
            from_index,
            limit,
        } => to_binary(&query_cw20_wallet_balances(
            deps, wallet, from_index, limit,
        )?),
    }
}

pub fn query_balances(
    deps: Deps,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> StdResult<BalancesResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    let from_index = from_index.unwrap_or_default();
    let limit = limit.unwrap_or(config.limit);

    let available_native_balance = AVAILABLE_NATIVE_BALANCE
        .range(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .map(|balance_res| balance_res.map(|(denom, amount)| Coin { denom, amount }))
        .collect::<StdResult<Vec<Coin>>>()?;

    let available_cw20_balance = AVAILABLE_CW20_BALANCE
        .range(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .map(|balance_res| {
            balance_res.map(|(address, amount)| Cw20CoinVerified { address, amount })
        })
        .collect::<StdResult<Vec<Cw20CoinVerified>>>()?;

    Ok(BalancesResponse {
        native_denom: config.native_denom,
        available_native_balance,
        available_cw20_balance,
    })
}

pub fn query_cw20_wallet_balances(
    deps: Deps,
    wallet: String,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> StdResult<Vec<Cw20CoinVerified>> {
    let config = CONFIG.load(deps.storage)?;
    let addr = deps.api.addr_validate(&wallet)?;
    let from_index = from_index.unwrap_or_default();
    let limit = limit.unwrap_or(config.limit);

    let balances = USERS_BALANCES_CW20
        .prefix(&addr)
        .range(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .map(|balance_res| {
            balance_res.map(|(address, amount)| Cw20CoinVerified { address, amount })
        })
        .collect::<StdResult<_>>()?;
    Ok(balances)
}
