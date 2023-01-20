use cosmwasm_std::{
    coin, Addr, Api, BankMsg, Binary, Coin, CosmosMsg, Deps, Empty, Env, GovMsg, IbcMsg,
    OverflowError, OverflowOperation::Sub, StakingMsg, StdError, SubMsg, SubMsgResult, Timestamp,
    Uint128, Uint64, WasmMsg,
};
use cron_schedule::Schedule;
use cw2::ContractVersion;
use cw20::{Cw20CoinVerified, Cw20ExecuteMsg};
use cw_rules_core::types::CroncatQuery;
use generic_query::PathToValue;
use hex::encode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::str::FromStr;

use crate::{
    error::CoreError,
    msg::{SimulateTaskResponse, TaskRequest},
    traits::{BalancesOperations, FindAndMutate, Intervals, ResultFailed},
};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct GenericBalance {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20CoinVerified>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum AgentStatus {
    // Default for any new agent, if tasks ratio allows
    Active,

    // Default for any new agent, until more tasks come online
    Pending,

    // More tasks are available, agent must checkin to become active
    Nominated,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Agent {
    // Where rewards get transferred
    pub payable_account_id: Addr,

    // accrued reward balance
    pub balance: GenericBalance,

    // stats
    pub total_tasks_executed: u64,

    // Holds slot number of the last slot when agent called proxy_call.
    // If agent does a task, this number is set to the current block.
    pub last_executed_slot: u64,

    // Timestamp of when agent first registered
    // Useful for rewarding agents for their patience while they are pending and operating service
    // Agent will be responsible to constantly monitor when it is their turn to join in active agent set (done as part of agent code loops)
    // Example data: 1633890060000000000 or 0
    pub register_start: Timestamp,
}

impl Agent {
    pub fn update(&mut self, last_executed_slot: u64) {
        self.total_tasks_executed = self.total_tasks_executed.saturating_add(1);
        self.last_executed_slot = last_executed_slot;
    }
}

/// Defines the spacing of execution
/// NOTES:
/// - Block Height Based: Once, Immediate, Block
/// - Timestamp Based: Cron
/// - No Epoch support directly, advised to use block heights instead
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum Interval {
    /// For when this is a non-recurring future scheduled TXN
    Once,

    /// The ugly batch schedule type, in case you need to exceed single TXN gas limits, within fewest block(s)
    Immediate,

    /// Allows timing based on block intervals rather than timestamps
    Block(u64),

    /// Crontab Spec String
    Cron(String),
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum Boundary {
    Height {
        start: Option<Uint64>,
        end: Option<Uint64>,
    },
    Time {
        start: Option<Timestamp>,
        end: Option<Timestamp>,
    },
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CheckedBoundary {
    pub start: Option<u64>,
    pub end: Option<u64>,
    pub is_block_boundary: Option<bool>,
}

impl CheckedBoundary {
    pub fn is_block_boundary(&self) -> bool {
        self.is_block_boundary.is_some() && self.is_block_boundary.unwrap()
    }

    pub fn new(boundary: Option<Boundary>, interval: &Interval) -> Result<Self, CoreError> {
        if let Some(boundary) = boundary {
            match (interval, boundary) {
                (Interval::Once | Interval::Cron(_), Boundary::Time { start, end }) => {
                    match (start, end) {
                        (Some(s), Some(e)) => {
                            if s.nanos() >= e.nanos() {
                                return Err(CoreError::InvalidBoundary {});
                            }
                            Ok(Self {
                                start: Some(s.nanos()),
                                end: Some(e.nanos()),
                                is_block_boundary: Some(false),
                            })
                        }
                        _ => Ok(Self {
                            start: start.map(|start| start.nanos()),
                            end: end.map(|end| end.nanos()),
                            is_block_boundary: Some(false),
                        }),
                    }
                }
                (
                    Interval::Once | Interval::Immediate | Interval::Block(_),
                    Boundary::Height { start, end },
                ) => match (start, end) {
                    (Some(s), Some(e)) => {
                        if s.u64() > e.u64() {
                            return Err(CoreError::InvalidBoundary {});
                        }
                        Ok(Self {
                            start: Some(s.u64()),
                            end: Some(e.u64()),
                            is_block_boundary: Some(true),
                        })
                    }
                    _ => Ok(Self {
                        start: start.map(Into::into),
                        end: end.map(Into::into),
                        is_block_boundary: Some(true),
                    }),
                },
                _ => Err(CoreError::InvalidBoundary {}),
            }
        } else {
            Ok(Self {
                start: None,
                end: None,
                is_block_boundary: Some(!matches!(interval, Interval::Cron(_))), //Boundary isnt provided, so default is block
            })
        }
    }
}

#[derive(
    Debug, PartialEq, Eq, std::hash::Hash, Deserialize, Serialize, Clone, Copy, JsonSchema,
)]
pub enum SlotType {
    Block,
    Cron,
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
// pub struct Rule {
//     /// TBD: Interchain query support (See ibc::IbcMsg)
//     // pub chain_id: Option<String>,

//     /// Account to direct all view calls against
//     pub contract_addr: String,

//     // NOTE: Only allow static pre-defined query msg
//     pub msg: Binary,
// }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Action<T = Empty> {
    // NOTE: Only allow static pre-defined query msg
    /// Supported CosmosMsgs only!
    pub msg: CosmosMsg<T>,

    /// The gas needed to safely process the execute msg
    pub gas_limit: Option<u64>,
}

impl Action {
    // Checking how much native coins sent in this action
    pub fn bank_sent(&self) -> Option<&[Coin]> {
        if let CosmosMsg::Bank(BankMsg::Send { amount, .. }) = &self.msg {
            Some(amount)
        } else {
            None
        }
    }

    // Checking how much cw20 coins sent in this action
    pub fn cw20_sent(&self, api: &dyn Api) -> Option<Cw20CoinVerified> {
        if let CosmosMsg::Wasm(WasmMsg::Execute {
            msg, contract_addr, ..
        }) = &self.msg
        {
            if let Ok(cw20_msg) = cosmwasm_std::from_binary(msg) {
                return match cw20_msg {
                    Cw20ExecuteMsg::Send { amount, .. } => Some(Cw20CoinVerified {
                        // unwraping safe here because we checked it at `is_valid_msg_calculate_usage`
                        address: api.addr_validate(contract_addr).unwrap(),
                        amount,
                    }),
                    Cw20ExecuteMsg::Transfer { amount, .. } => Some(Cw20CoinVerified {
                        address: api.addr_validate(contract_addr).unwrap(),
                        amount,
                    }),
                    _ => None,
                };
            }
        }
        None
    }
}

/// The response required by all rule queries. Bool is needed for croncat, T allows flexible rule engine
pub type RuleResponse<T> = (bool, T);

impl TaskRequest {
    /// Validate the task actions only use the supported messages
    /// We're iterating over all actions
    /// so it's a great place for calculaing balance usages
    // Consider moving Config to teh cw-croncat-core so this method can take reference of that
    #[allow(clippy::too_many_arguments)]
    pub fn is_valid_msg_calculate_usage(
        &self,
        api: &dyn Api,
        self_addr: &Addr,
        sender: &Addr,
        owner_id: &Addr,
        base_gas: u64,
        action_gas: u64,
        query_gas: u64,
        wasm_query_gas: u64,
    ) -> Result<(GenericBalance, u64), CoreError> {
        let mut gas_amount: u64 = base_gas;
        let mut amount_for_one_task = GenericBalance::default();

        if self.actions.is_empty() {
            return Err(CoreError::InvalidAction {});
        }
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

        if let Some(queries) = self.queries.as_ref() {
            // If task has queries - Rules contract is queried which is wasm query
            gas_amount = gas_amount
                .checked_add(wasm_query_gas)
                .ok_or(CoreError::InvalidWasmMsg {})?;
            for query in queries.iter() {
                match query {
                    CroncatQuery::HasBalanceGte(_) => {
                        gas_amount = gas_amount
                            .checked_add(query_gas)
                            .ok_or(CoreError::InvalidWasmMsg {})?;
                    }
                    _ => {
                        gas_amount = gas_amount
                            .checked_add(wasm_query_gas)
                            .ok_or(CoreError::InvalidWasmMsg {})?;
                    }
                }
            }
        }
        Ok((amount_for_one_task, gas_amount))
    }

    pub fn as_task(
        &self,
        env: &Env,
        deps: &Deps,
        contract_info: &ContractInfo,
        funds: Vec<Coin>,
        economics: EconomicsContext,
    ) -> Result<Task, CoreError> {
        let sender = deps
            .api
            .addr_validate(&self.sender.clone().unwrap_or_else(|| "ADDRESS".to_owned()))?;

        let cw20 = if !self.cw20_coins.is_empty() {
            let mut cw20: Vec<Cw20CoinVerified> = Vec::with_capacity(self.cw20_coins.len());
            for coin in &self.cw20_coins {
                cw20.push(Cw20CoinVerified {
                    address: deps.api.addr_validate(&coin.address)?,
                    amount: coin.amount,
                })
            }
            cw20
        } else {
            vec![]
        };
        let boundary = CheckedBoundary::new(self.boundary, &self.interval)?;

        let (mut amount_for_one_task, gas_amount) = self.is_valid_msg_calculate_usage(
            deps.api,
            &env.contract.address,
            &sender,
            contract_info.owner_addr,
            economics.gas_base_fee,
            economics.gas_action_fee,
            economics.query_gas,
            economics.wasm_query_gas,
        )?;
        let gas_amount_with_agent_fee = gas_amount_with_agent_fee(gas_amount, economics.agent_fee)?;
        let price = economics.gas_price.calculate(gas_amount_with_agent_fee)?;
        amount_for_one_task
            .native
            .find_checked_add(&coin(price, economics.native_denom))?;

        Ok(Task {
            owner_id: sender,
            interval: self.interval.clone(),
            boundary,
            stop_on_fail: self.stop_on_fail,
            total_deposit: GenericBalance {
                native: funds,
                cw20,
            },
            amount_for_one_task,
            actions: self.actions.clone(),
            queries: self.queries.clone(),
            transforms: self.transforms.clone(),
            version: contract_info.version.version.clone(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Task {
    /// Entity responsible for this task, can change task details
    pub owner_id: Addr,

    /// Scheduling definitions
    pub interval: Interval,
    pub boundary: CheckedBoundary,

    /// Defines if this task can continue until balance runs out
    pub stop_on_fail: bool,

    /// NOTE: Only tally native balance here, manager can maintain token/balances outside of tasks
    pub total_deposit: GenericBalance,

    pub amount_for_one_task: GenericBalance,

    /// The cosmos message to call, if time or rules are met
    pub actions: Vec<Action>,
    /// A prioritized list of messages that can be chained decision matrix
    /// required to complete before task action
    /// Rules MUST return the ResolverResponse type
    pub queries: Option<Vec<CroncatQuery>>,
    pub transforms: Option<Vec<Transform>>,
    // TODO: funds! should we support funds being attached?
    pub version: String,
}

impl Task {
    /// Get the hash of a task based on parameters
    pub fn to_hash(&self) -> String {
        let message = format!(
            "{:?}{:?}{:?}{:?}{:?}{:?}",
            self.owner_id,
            self.interval,
            self.boundary,
            self.actions,
            self.queries,
            self.transforms
        );

        let hash = Sha256::digest(message.as_bytes());
        encode(hash)
    }
    /// Get the hash of a task based on parameters
    pub fn to_hash_vec(&self) -> Vec<u8> {
        self.to_hash().into_bytes()
    }

    pub fn verify_enough_balances(&self, recurring: bool) -> Result<(), CoreError> {
        let multiplier = Uint128::from(if recurring { 2u128 } else { 1u128 });

        self.verify_enough_native(multiplier)?;
        self.verify_enough_cw20(multiplier)?;
        Ok(())
    }

    pub fn verify_enough_cw20(&self, multiplier: Uint128) -> Result<(), CoreError> {
        for coin in self.amount_for_one_task.cw20.iter() {
            if let Some(balance) = self
                .total_deposit
                .cw20
                .iter()
                .find(|balance| balance.address == coin.address)
            {
                if balance.amount < coin.amount * multiplier {
                    return Err(CoreError::NotEnoughCw20 {
                        addr: coin.address.to_string(),
                        lack: coin.amount * multiplier - balance.amount,
                    });
                }
            } else {
                return Err(CoreError::NotEnoughCw20 {
                    addr: coin.address.to_string(),
                    lack: coin.amount,
                });
            }
        }
        Ok(())
    }

    pub fn verify_enough_native(&self, multiplier: Uint128) -> Result<(), CoreError> {
        for coin in self.amount_for_one_task.native.iter() {
            if let Some(balance) = self
                .total_deposit
                .native
                .iter()
                .find(|balance| balance.denom == coin.denom)
            {
                if balance.amount < coin.amount * multiplier {
                    return Err(CoreError::NotEnoughNative {
                        denom: coin.denom.clone(),
                        lack: coin.amount * multiplier - balance.amount,
                    });
                }
            } else {
                return Err(CoreError::NotEnoughNative {
                    denom: coin.denom.clone(),
                    lack: coin.amount * multiplier,
                });
            }
        }
        Ok(())
    }

    /// Get task gas total
    /// helper for getting total configured gas for this tasks actions
    pub fn get_submsgs_with_total_gas(
        &self,
        base_gas: u64,
        action_gas: u64,
        query_gas: u64,
        wasm_query_gas: u64,
        next_idx: u64,
    ) -> Result<(Vec<SubMsg<Empty>>, u64), CoreError> {
        let mut gas: u64 = base_gas;
        let mut sub_msgs = Vec::with_capacity(self.actions.len());
        for action in self.actions.iter() {
            gas = gas
                .checked_add(action.gas_limit.unwrap_or(action_gas))
                .ok_or(CoreError::InvalidGas {})?;
            let sub_msg: SubMsg = SubMsg::reply_always(action.msg.clone(), next_idx);
            if let Some(gas_limit) = action.gas_limit {
                sub_msgs.push(sub_msg.with_gas_limit(gas_limit));
            } else {
                sub_msgs.push(sub_msg);
            }
        }

        if let Some(queries) = self.queries.as_ref() {
            // If task has queries - Rules contract is queried which is wasm query
            gas = gas
                .checked_add(wasm_query_gas)
                .ok_or(CoreError::InvalidGas {})?;
            for query in queries.iter() {
                match query {
                    CroncatQuery::HasBalanceGte(_) => {
                        gas = gas.checked_add(query_gas).ok_or(CoreError::InvalidGas {})?;
                    }
                    _ => {
                        gas = gas
                            .checked_add(wasm_query_gas)
                            .ok_or(CoreError::InvalidGas {})?;
                    }
                }
            }
        }
        Ok((sub_msgs, gas))
    }

    /// Calculate gas usage for this task
    // pub fn calculate_gas_usage(
    //     &self,
    //     cfg: &Config,
    //     actions: &[Action],
    // ) -> Result<Coin, ContractError> {
    //     let mut gas_used = 0;
    //     for action in actions {
    //         if let Some(gas_limit) = action.gas_limit {
    //             gas_used += gas_limit;
    //         } else {
    //             gas_used += cfg.gas_base_fee;
    //         }
    //     }
    //     let gas_amount = calculate_required_amount(gas_used, cfg.agent_fee)?;
    //     let price_amount = cfg.gas_fraction.calculate(gas_amount, 1)?;
    //     let price = coin(price_amount, &cfg.native_denom);
    //     Ok(price)
    // }

    /// Get whether the task is with rules
    pub fn with_queries(&self) -> bool {
        self.queries
            .as_ref()
            .map_or(false, |queries| !queries.is_empty())
    }

    /// Check if given Addr is the owner
    pub fn is_owner(&self, addr: Addr) -> bool {
        self.owner_id == addr
    }

    /// Replace `RULE_RES_PLACEHOLDER` to the result value from the rules
    /// Recalculate cw20 usage if any replacements
    pub fn replace_values(
        &mut self,
        api: &dyn Api,
        cron_addr: &Addr,
        task_hash: &str,
        construct_res_data: Vec<cosmwasm_std::Binary>,
    ) -> Result<(), CoreError> {
        if let Some(ref transforms) = self.transforms {
            for transform in transforms {
                let wasm_msg = self
                    .actions
                    .get_mut(transform.action_idx as usize)
                    .and_then(|action| {
                        if let CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: _,
                            msg,
                            funds: _,
                        }) = &mut action.msg
                        {
                            Some(msg)
                        } else {
                            None
                        }
                    })
                    .ok_or(CoreError::TaskNoLongerValid {
                        task_hash: task_hash.to_owned(),
                    })?;
                let mut action_value = cosmwasm_std::from_binary(wasm_msg)?;

                let mut q_val = construct_res_data
                    .get(transform.query_idx as usize)
                    .ok_or(CoreError::TaskNoLongerValid {
                        task_hash: task_hash.to_owned(),
                    })
                    .and_then(|binary| cosmwasm_std::from_binary(binary).map_err(Into::into))?;
                let replace_value = transform.query_response_path.find_value(&mut q_val)?;
                let replaced_value = transform.action_path.find_value(&mut action_value)?;
                *replaced_value = replace_value.clone();
                *wasm_msg = Binary(
                    serde_json_wasm::to_vec(&action_value)
                        .map_err(|e| CoreError::Std(StdError::generic_err(e.to_string())))?,
                );
            }
            let cw20_amount_recalculated =
                self.recalculate_cw20_usage(api, cron_addr, task_hash)?;
            self.amount_for_one_task.cw20 = cw20_amount_recalculated;
            if self.verify_enough_cw20(1u128.into()).is_err() {
                return Err(CoreError::TaskNoLongerValid {
                    task_hash: task_hash.to_owned(),
                });
            };
        }
        Ok(())
    }

    fn recalculate_cw20_usage(
        &self,
        api: &dyn Api,
        cron_addr: &Addr,
        task_hash: &str,
    ) -> Result<Vec<Cw20CoinVerified>, CoreError> {
        let actions = self.actions.iter();
        let mut cw20_coins = vec![];
        for action in actions {
            if let CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr, msg, ..
            }) = &action.msg
            {
                if cron_addr.as_str().eq(contract_addr) {
                    return Err(CoreError::TaskNoLongerValid {
                        task_hash: task_hash.to_owned(),
                    });
                }
                if let Ok(cw20_msg) = cosmwasm_std::from_binary(msg) {
                    match cw20_msg {
                        Cw20ExecuteMsg::Send { amount, .. } if !amount.is_zero() => cw20_coins
                            .find_checked_add(&Cw20CoinVerified {
                                address: api.addr_validate(contract_addr)?,
                                amount,
                            })?,
                        Cw20ExecuteMsg::Transfer { amount, .. } if !amount.is_zero() => cw20_coins
                            .find_checked_add(&Cw20CoinVerified {
                                address: api.addr_validate(contract_addr)?,
                                amount,
                            })?,
                        _ => {
                            return Err(CoreError::TaskNoLongerValid {
                                task_hash: task_hash.to_owned(),
                            });
                        }
                    }
                }
            }
        }
        Ok(cw20_coins)
    }
}

pub fn simulate_task(
    env: &Env,
    deps: &Deps,
    task: TaskRequest,
    funds: Option<Vec<Coin>>,
    economics: EconomicsContext,
    contract_info: &ContractInfo,
    slot_granularity_time: u64,
) -> Result<SimulateTaskResponse, CoreError> {
    let funds = funds.unwrap_or_default();
    let task_info = task
        .as_task(env, deps, contract_info, funds.clone(), economics.clone())
        .unwrap();
    let task_hash = task_info.to_hash();

    // Calculate expected gas
    let gas_amount = calculate_gas(task_info.clone(), economics)?;

    // If task has queries, return 1 occurrence
    if task_info.with_queries() {
        return Ok(SimulateTaskResponse {
            estimated_gas: gas_amount,
            occurrences: 1,
            task_hash,
        });
    }

    // Calculate the maximum amount of occurrences for given funds (if funds are provided)
    // It is defined by the amount for one execution
    let occurrences_for_funds = if !funds.is_empty() {
        let amount_for_one_task_native = &task_info.amount_for_one_task.native.first().unwrap();
        let amount_given = funds
            .iter()
            .find(|coin| coin.denom == amount_for_one_task_native.denom)
            .ok_or_else(|| {
                CoreError::Std(StdError::generic_err(format!(
                    "No coins with correct denom: {}",
                    amount_for_one_task_native.denom
                )))
            })?
            .amount
            .u128();
        Some(
            amount_given
                .checked_div(amount_for_one_task_native.amount.u128())
                .unwrap() as u64,
        )
    } else {
        None
    };

    // Calculate the maximum amount of occurrences according to the given interval and boundary
    // and compare with occurrences_for_funds, if it's defined
    let interval = task.interval;
    let boundary = CheckedBoundary::new(task.boundary, &interval)?;

    let occurrences = match interval {
        Interval::Once | Interval::Immediate => 1,
        Interval::Block(block) => {
            if boundary.end.is_some() {
                let mut next_id = env.block.height;
                let mut occur: u64 = 0;

                while next_id > 0 {
                    let prev_id = next_id;
                    (next_id, _) = get_next_block_by_offset(next_id, boundary, block);
                    occur = occur
                        .checked_add(1)
                        .ok_or_else(|| StdError::generic_err("Invalid amount of occurrences"))?;

                    // If this was the last occurrence before the end, get_next_block_by_offset returns the same block again
                    // If the block number recur, stop the loop
                    if prev_id == next_id {
                        break;
                    }
                }
                // The last occurrence with next_id == 0 has to be subtracted
                let occurrences_for_boundary = occur - 1;

                // If funds are provided, take the minimum of calculated occurrences
                // If funds aren't provided, returns occurrences_for_boundary
                std::cmp::min(
                    occurrences_for_boundary,
                    occurrences_for_funds.unwrap_or(u64::MAX),
                )
            } else {
                // If there's no end boundary, the occurrences are limited only by funds
                // If there's no funds, return 1
                occurrences_for_funds.unwrap_or(1)
            }
        }
        Interval::Cron(crontab) => {
            // get_next_cron_time
            if boundary.end.is_some() {
                let mut next_id = env.block.time.nanos();
                let mut occur: u64 = 0;

                while next_id > 0 {
                    let prev_id = next_id;
                    (next_id, _) = get_next_cron_time_by_ts(
                        next_id,
                        boundary,
                        &crontab,
                        slot_granularity_time,
                    );
                    occur = occur
                        .checked_add(1)
                        .ok_or_else(|| StdError::generic_err("Invalid amount of occurrences"))?;

                    // If this was the last occurrence before the end, get_next_block_by_offset returns the same block again
                    // If the block number recur, stop the loop
                    if prev_id == next_id {
                        break;
                    }
                }
                // The last occurrence with next_id == 0 has to be subtracted
                let occurrences_for_boundary = occur - 1;

                // If funds are provided, take the minimum of calculated occurrences
                // If funds aren't provided, returns occurrences_for_boundary
                std::cmp::min(
                    occurrences_for_boundary,
                    occurrences_for_funds.unwrap_or(u64::MAX),
                )
            } else {
                // If there's no end boundary, the occurrences are limited only by funds
                // If there's no funds, return 1
                occurrences_for_funds.unwrap_or(1)
            }
        }
    };

    Ok(SimulateTaskResponse {
        estimated_gas: gas_amount,
        occurrences,
        task_hash,
    })
}

// Does the same as get_next_cron_time but for specific timestamp
fn get_next_cron_time_by_ts(
    current_ts: u64,
    boundary: CheckedBoundary,
    crontab: &str,
    slot_granularity_time: u64,
) -> (u64, SlotType) {
    let current_ts_slot = current_ts.saturating_sub(current_ts % slot_granularity_time);

    // get earliest possible time
    let current_ts = match boundary.start {
        Some(ts) if current_ts < ts => ts,
        _ => current_ts,
    };

    // receive time from schedule, calculate slot for this time
    let schedule = Schedule::from_str(crontab).unwrap();
    let next_ts = schedule.next_after(&current_ts).unwrap();
    let next_ts_slot = next_ts.saturating_sub(next_ts % slot_granularity_time);

    // put task in the next slot if next_ts_slot in the current slot
    let next_slot = if next_ts_slot == current_ts_slot {
        next_ts_slot + slot_granularity_time
    } else {
        next_ts_slot
    };

    match boundary.end {
        Some(end) if current_ts > end => (0, SlotType::Cron),
        Some(end) => {
            let end_slot = end.saturating_sub(end % slot_granularity_time);
            (u64::min(end_slot, next_slot), SlotType::Cron)
        }
        _ => (next_slot, SlotType::Cron),
    }
}

fn calculate_gas(task: Task, economics: EconomicsContext) -> Result<u64, CoreError> {
    let mut gas_amount: u64 = economics.gas_base_fee;
    let gas_action_fee = economics.gas_action_fee;

    // Gas for actions:
    for action in task.actions.iter() {
        gas_amount = gas_amount
            .checked_add(action.gas_limit.unwrap_or(gas_action_fee))
            .ok_or(CoreError::NoGasLimit {})?;
    }
    // Gas for queries
    if let Some(queries) = task.queries.as_ref() {
        // If task has queries - Rules contract is queried which is wasm query
        gas_amount = gas_amount
            .checked_add(economics.wasm_query_gas)
            .ok_or(CoreError::InvalidWasmMsg {})?;
        for query in queries.iter() {
            match query {
                CroncatQuery::HasBalanceGte(_) => {
                    gas_amount = gas_amount
                        .checked_add(economics.query_gas)
                        .ok_or(CoreError::InvalidWasmMsg {})?;
                }
                _ => {
                    gas_amount = gas_amount
                        .checked_add(economics.wasm_query_gas)
                        .ok_or(CoreError::InvalidWasmMsg {})?;
                }
            }
        }
    };
    Ok(gas_amount)
}

/// Calculate the amount including agent_fee
pub fn gas_amount_with_agent_fee(gas_amount: u64, agent_fee: u64) -> Result<u64, CoreError> {
    gas_amount
        .checked_mul(agent_fee)
        .and_then(|n| n.checked_div(100))
        .and_then(|n| n.checked_add(gas_amount))
        .ok_or(CoreError::InvalidGas {})
}

impl FindAndMutate<'_, Coin> for Vec<Coin> {
    fn find_checked_add(&mut self, add: &Coin) -> Result<(), CoreError> {
        let token = self.iter_mut().find(|exist| exist.denom == add.denom);
        match token {
            Some(exist) => {
                exist.amount = exist
                    .amount
                    .checked_add(add.amount)
                    .map_err(StdError::overflow)?
            }
            None => self.push(add.clone()),
        }
        Ok(())
    }

    fn find_checked_sub(&mut self, sub: &Coin) -> Result<(), CoreError> {
        let coin = self.iter().position(|exist| exist.denom == sub.denom);
        match coin {
            Some(exist) => {
                match self[exist].amount.cmp(&sub.amount) {
                    std::cmp::Ordering::Less => {
                        return Err(CoreError::Std(StdError::overflow(OverflowError::new(
                            Sub,
                            self[exist].amount,
                            sub.amount,
                        ))))
                    }
                    std::cmp::Ordering::Equal => {
                        self.swap_remove(exist);
                    }
                    std::cmp::Ordering::Greater => self[exist].amount -= sub.amount,
                };
                Ok(())
            }
            None => Err(CoreError::EmptyBalance {}),
        }
    }
}

impl FindAndMutate<'_, Cw20CoinVerified> for Vec<Cw20CoinVerified> {
    fn find_checked_add(&mut self, add: &Cw20CoinVerified) -> Result<(), CoreError> {
        let token = self.iter_mut().find(|exist| exist.address == add.address);
        match token {
            Some(exist) => {
                exist.amount = exist
                    .amount
                    .checked_add(add.amount)
                    .map_err(StdError::overflow)?
            }
            None => self.push(add.clone()),
        }
        Ok(())
    }

    fn find_checked_sub(&mut self, sub: &Cw20CoinVerified) -> Result<(), CoreError> {
        let coin_p = self.iter().position(|exist| exist.address == sub.address);
        match coin_p {
            Some(exist) => {
                match self[exist].amount.cmp(&sub.amount) {
                    std::cmp::Ordering::Less => {
                        return Err(CoreError::Std(StdError::overflow(OverflowError::new(
                            Sub,
                            self[exist].amount,
                            sub.amount,
                        ))))
                    }
                    std::cmp::Ordering::Equal => {
                        self.swap_remove(exist);
                    }
                    std::cmp::Ordering::Greater => self[exist].amount -= sub.amount,
                };
                Ok(())
            }
            None => Err(CoreError::EmptyBalance {}),
        }
    }
}

impl<'a, T, Rhs> BalancesOperations<'a, T, Rhs> for Vec<T>
where
    Rhs: IntoIterator<Item = &'a T>,
    Self: FindAndMutate<'a, T>,
    T: 'a,
{
    fn checked_add_coins(&mut self, add: Rhs) -> Result<(), CoreError> {
        for add_token in add {
            self.find_checked_add(add_token)?;
        }
        Ok(())
    }

    fn checked_sub_coins(&mut self, sub: Rhs) -> Result<(), CoreError> {
        for sub_token in sub {
            self.find_checked_sub(sub_token)?;
        }
        Ok(())
    }
}

impl GenericBalance {
    pub fn checked_add_native(&mut self, add: &[Coin]) -> Result<(), CoreError> {
        self.native.checked_add_coins(add)
    }

    pub fn checked_add_cw20(&mut self, add: &[Cw20CoinVerified]) -> Result<(), CoreError> {
        self.cw20.checked_add_coins(add)
    }

    pub fn checked_sub_native(&mut self, sub: &[Coin]) -> Result<(), CoreError> {
        self.native.checked_sub_coins(sub)
    }

    pub fn checked_sub_cw20(&mut self, sub: &[Cw20CoinVerified]) -> Result<(), CoreError> {
        self.cw20.checked_sub_coins(sub)
    }

    pub fn checked_sub_generic(&mut self, sub: &GenericBalance) -> Result<(), CoreError> {
        self.checked_sub_native(&sub.native)?;
        self.checked_sub_cw20(&sub.cw20)
    }
}

impl ResultFailed for SubMsgResult {
    fn failed(&self) -> bool {
        match self {
            SubMsgResult::Ok(response) => response.events.iter().any(|event| {
                event.attributes.iter().any(|attribute| {
                    event.ty == "reply"
                        && attribute.key == "mode"
                        && attribute.value == "handle_failure"
                })
            }),
            SubMsgResult::Err(_) => true,
        }
    }
}

// Get the next block within the boundary
fn get_next_block_limited(env: &Env, boundary: CheckedBoundary) -> (u64, SlotType) {
    let current_block_height = env.block.height;

    let next_block_height = match boundary.start {
        // shorthand - remove 1 since it adds 1 later
        Some(id) if current_block_height < id => id - 1,
        _ => current_block_height,
    };

    match boundary.end {
        // stop if passed end height
        Some(end) if current_block_height > end => (0, SlotType::Block),

        // we ONLY want to catch if we're passed the end block height
        Some(end) if next_block_height > end => (end, SlotType::Block),

        // immediate needs to return this block + 1
        _ => (next_block_height + 1, SlotType::Block),
    }
}

// So either:
// - Boundary specifies a start/end that block offsets can be computed from
// - Block offset will truncate to specific modulo offsets
pub(crate) fn get_next_block_by_offset(
    block_height: u64,
    boundary: CheckedBoundary,
    interval: u64,
) -> (u64, SlotType) {
    let current_block_height = block_height;
    let modulo_block =
        current_block_height.saturating_sub(current_block_height % interval) + interval;

    let next_block_height = match boundary.start {
        Some(start) if current_block_height < start => {
            let rem = start % interval;
            if rem > 0 {
                start.saturating_sub(rem) + interval
            } else {
                start
            }
        }
        _ => modulo_block,
    };

    match boundary.end {
        // stop if passed end height
        Some(end) if current_block_height > end => (0, SlotType::Block),

        // we ONLY want to catch if we're passed the end block height
        Some(end) => {
            let end_height = if let Some(rem) = end.checked_rem(interval) {
                end.saturating_sub(rem)
            } else {
                end
            };
            // we ONLY want to catch if we're passed the end block height
            (
                std::cmp::min(next_block_height, end_height),
                SlotType::Block,
            )
        }

        None => (next_block_height, SlotType::Block),
    }
}

// Get the slot number (in nanos) of the next task according to boundaries
// Unless current slot is the end slot, don't put in the current slot
fn get_next_cron_time(
    env: &Env,
    boundary: CheckedBoundary,
    crontab: &str,
    slot_granularity_time: u64,
) -> (u64, SlotType) {
    let current_block_ts = env.block.time.nanos();
    let current_block_slot =
        current_block_ts.saturating_sub(current_block_ts % slot_granularity_time);

    // get earliest possible time
    let current_ts = match boundary.start {
        Some(ts) if current_block_ts < ts => ts,
        _ => current_block_ts,
    };

    // receive time from schedule, calculate slot for this time
    let schedule = Schedule::from_str(crontab).unwrap();
    let next_ts = schedule.next_after(&current_ts).unwrap();
    let next_ts_slot = next_ts.saturating_sub(next_ts % slot_granularity_time);

    // put task in the next slot if next_ts_slot in the current slot
    let next_slot = if next_ts_slot == current_block_slot {
        next_ts_slot + slot_granularity_time
    } else {
        next_ts_slot
    };

    match boundary.end {
        Some(end) if current_block_ts > end => (0, SlotType::Cron),
        Some(end) => {
            let end_slot = end.saturating_sub(end % slot_granularity_time);
            (u64::min(end_slot, next_slot), SlotType::Cron)
        }
        _ => (next_slot, SlotType::Cron),
    }
}

impl Intervals for Interval {
    fn next(
        &self,
        env: &Env,
        boundary: CheckedBoundary,
        slot_granularity_time: u64,
    ) -> (u64, SlotType) {
        match self {
            // If Once, return the first block within a specific range that can be triggered 1 time.
            // If Immediate, return the first block within a specific range that can be triggered immediately, potentially multiple times.
            Interval::Once | Interval::Immediate => {
                if boundary.is_block_boundary() {
                    get_next_block_limited(env, boundary)
                } else {
                    get_next_cron_time(env, boundary, "0 0 * * * *", slot_granularity_time)
                }
            }
            // return the first block within a specific range that can be triggered 1 or more times based on timestamps.
            // Uses crontab spec
            Interval::Cron(crontab) => {
                get_next_cron_time(env, boundary, crontab, slot_granularity_time)
            }
            // return the block within a specific range that can be triggered 1 or more times based on block heights.
            // Uses block offset (Example: Block(100) will trigger every 100 blocks)
            // So either:
            // - Boundary specifies a start/end that block offsets can be computed from
            // - Block offset will truncate to specific modulo offsets
            Interval::Block(block) => get_next_block_by_offset(env.block.height, boundary, *block),
        }
    }

    fn is_valid(&self) -> bool {
        match self {
            Interval::Once => true,
            Interval::Immediate => true,
            Interval::Block(_) => true,
            Interval::Cron(crontab) => {
                let s = Schedule::from_str(crontab);
                s.is_ok()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct GasPrice {
    pub numerator: u64,
    pub denominator: u64,
    /// Note
    pub gas_adjustment_numerator: u64,
}

impl GasPrice {
    pub fn is_valid(&self) -> bool {
        self.denominator != 0 && self.numerator != 0 && self.gas_adjustment_numerator != 0
    }

    pub fn calculate(&self, gas_amount: u64) -> Result<u128, CoreError> {
        let gas_adjusted = gas_amount
            .checked_mul(self.gas_adjustment_numerator)
            .and_then(|g| g.checked_div(self.denominator))
            .ok_or(CoreError::InvalidGas {})?;

        let price = gas_adjusted
            .checked_mul(self.numerator)
            .and_then(|g| g.checked_div(self.denominator))
            .ok_or(CoreError::InvalidGas {})?;

        Ok(price as u128)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Transform {
    pub action_idx: u64,
    pub query_idx: u64,
    pub action_path: PathToValue,
    pub query_response_path: PathToValue,
}
#[derive(Clone)]
pub struct EconomicsContext<'a> {
    pub gas_base_fee: u64,
    pub gas_action_fee: u64,
    pub agent_fee: u64,
    pub native_denom: &'a str,
    pub gas_price: &'a GasPrice,
    pub query_gas: u64,
    pub wasm_query_gas: u64,
}
pub struct ContractInfo<'a> {
    pub addr: &'a Addr,
    pub version: &'a ContractVersion,
    pub owner_addr: &'a Addr,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum BalancerMode {
    ActivationOrder,
    Equalizer,
}
