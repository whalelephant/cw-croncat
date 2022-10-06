use crate::error::ContractError;
use crate::helpers::GenericBalance;
use crate::state::{Config, CwCroncat};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;
use cw_croncat_core::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cw_croncat_core::traits::ResultFailed;
use cw_croncat_core::types::SlotType;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-croncat";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_NOMINATION_DURATION: u16 = 360;

// default for juno
pub(crate) const GAS_BASE_FEE_JUNO: u64 = 400_000;

// #[cfg(not(feature = "library"))]
impl<'a> CwCroncat<'a> {
    pub fn instantiate(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        // keep tally of balances initialized
        let available_balance = GenericBalance {
            native: info.funds,
            cw20: Default::default(),
        };

        let owner_id = if let Some(owner_id) = msg.owner_id {
            deps.api.addr_validate(&owner_id)?
        } else {
            info.sender
        };

        let gas_base_fee = if let Some(base_fee) = msg.gas_base_fee {
            base_fee.u64()
        } else {
            GAS_BASE_FEE_JUNO
        };

        let config = Config {
            paused: false,
            owner_id,
            // treasury_id: None,
            min_tasks_per_agent: 3,
            agent_active_indices: vec![(SlotType::Block, 0, 0), (SlotType::Cron, 0, 0)],
            agents_eject_threshold: 600, // how many slots an agent can miss before being ejected. 10 * 60 = 1hr
            available_balance,
            staked_balance: GenericBalance::default(),
            agent_fee: Coin::new(5, msg.denom.clone()), // TODO: CHANGE AMOUNT HERE!!! 0.0005 Juno (2000 tasks = 1 Juno)
            gas_price: 1,
            proxy_callback_gas: 3,
            gas_base_fee,
            slot_granularity: 60_000_000_000,
            native_denom: msg.denom,
            cw20_whitelist: vec![],
            // TODO: ????
            // cw20_fees: vec![],
            agent_nomination_duration: msg
                .agent_nomination_duration
                .unwrap_or(DEFAULT_NOMINATION_DURATION),
            limit: 100,
            cw_rules_addr: cosmwasm_std::Addr::unchecked(&msg.cw_rules_addr), // deps.api.addr_validate(&msg.cw_rules_addr)?,
        };
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        self.config.save(deps.storage, &config)?;
        self.agent_active_queue
            .save(deps.storage, &Default::default())?;
        self.agent_pending_queue
            .save(deps.storage, &Default::default())?;
        self.task_total.save(deps.storage, &Default::default())?;
        self.reply_index.save(deps.storage, &Default::default())?;
        self.agent_nomination_begin_time.save(deps.storage, &None)?;
        self.tasks_with_rules_total.save(deps.storage, &0)?;

        // all instantiated data
        Ok(Response::new()
            .add_attribute("method", "instantiate")
            .add_attribute("paused", config.paused.to_string())
            .add_attribute("owner_id", config.owner_id.to_string())
            // .add_attribute(
            //     "treasury_id",
            //     config
            //         .treasury_id
            //         .unwrap_or_else(|| Addr::unchecked(""))
            //         .to_string(),
            // )
            .add_attribute(
                "min_tasks_per_agent",
                config.min_tasks_per_agent.to_string(),
            )
            .add_attribute(
                "agent_active_indices",
                config
                    .agent_active_indices
                    .iter()
                    .map(|a| format!("{:?}.{}", a.0, a.1))
                    .collect::<String>(),
            )
            .add_attribute(
                "agents_eject_threshold",
                config.agents_eject_threshold.to_string(),
            )
            .add_attribute("native_denom", config.native_denom)
            .add_attribute("agent_fee", config.agent_fee.to_string())
            .add_attribute("gas_price", config.gas_price.to_string())
            .add_attribute("proxy_callback_gas", config.proxy_callback_gas.to_string())
            .add_attribute("slot_granularity", config.slot_granularity.to_string()))
    }

    pub fn execute(
        &mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::UpdateSettings { .. } => self.update_settings(deps, info, msg),
            ExecuteMsg::MoveBalances {
                balances,
                account_id,
            } => self.move_balances(deps, info, env, balances, account_id),

            ExecuteMsg::RegisterAgent { payable_account_id } => {
                self.register_agent(deps, info, env, payable_account_id)
            }
            ExecuteMsg::UpdateAgent { payable_account_id } => {
                self.update_agent(deps, info, env, payable_account_id)
            }
            ExecuteMsg::UnregisterAgent {} => self.unregister_agent(deps, info, env),
            ExecuteMsg::WithdrawReward {} => self.withdraw_agent_balance(deps, info, env),
            ExecuteMsg::CheckInAgent {} => self.accept_nomination_agent(deps, info, env),

            ExecuteMsg::CreateTask { task } => self.create_task(deps, info, env, task),
            ExecuteMsg::RemoveTask { task_hash } => {
                self.remove_task(deps.storage, task_hash, Some(info))
            }
            ExecuteMsg::RefillTaskBalance { task_hash } => self.refill_task(deps, info, task_hash),
            ExecuteMsg::RefillTaskCw20Balance {
                task_hash,
                cw20_coins,
            } => self.refill_task_cw20(deps, info, task_hash, cw20_coins),
            ExecuteMsg::ProxyCall {
                task_hash: Some(task_hash),
            } => self.proxy_call_with_rules(deps, info, env, task_hash),
            ExecuteMsg::ProxyCall { task_hash: None } => self.proxy_call(deps, info, env),
            ExecuteMsg::Receive(msg) => self.receive_cw20(deps, info, msg),
            ExecuteMsg::WithdrawWalletBalance {
                cw20_amounts: cw20_balances,
            } => self.withdraw_wallet_balances(deps, info, cw20_balances),
        }
    }

    pub fn query(&mut self, deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::GetConfig {} => to_binary(&self.query_config(deps)?),
            QueryMsg::GetBalances {} => to_binary(&self.query_balances(deps)?),

            QueryMsg::GetAgent { account_id } => {
                to_binary(&self.query_get_agent(deps, env, account_id)?)
            }
            QueryMsg::GetAgentIds {} => to_binary(&self.query_get_agent_ids(deps)?),
            QueryMsg::GetAgentTasks { account_id } => {
                to_binary(&self.query_get_agent_tasks(deps, env, account_id)?)
            }

            QueryMsg::GetTasks { from_index, limit } => {
                to_binary(&self.query_get_tasks(deps, from_index, limit)?)
            }
            QueryMsg::GetTasksWithRules { from_index, limit } => {
                to_binary(&self.query_get_tasks_with_rules(deps, from_index, limit)?)
            }
            QueryMsg::GetTasksByOwner { owner_id } => {
                to_binary(&self.query_get_tasks_by_owner(deps, owner_id)?)
            }
            QueryMsg::GetTask { task_hash } => to_binary(&self.query_get_task(deps, task_hash)?),
            QueryMsg::GetTaskHash { task } => to_binary(&self.query_get_task_hash(*task)?),
            QueryMsg::ValidateInterval { interval } => {
                to_binary(&self.query_validate_interval(interval)?)
            }
            QueryMsg::GetSlotHashes { slot } => to_binary(&self.query_slot_tasks(deps, slot)?),
            QueryMsg::GetSlotIds {} => to_binary(&self.query_slot_ids(deps)?),
            QueryMsg::GetWalletBalances { wallet } => {
                to_binary(&self.query_wallet_balances(deps, wallet)?)
            }
            QueryMsg::GetState { from_index, limit } => {
                to_binary(&self.get_state(deps, from_index, limit)?)
            }
        }
    }

    pub fn reply(&self, deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
        // Route the next fns with the reply queue id meta
        let queue_item = self
            .reply_queue
            .may_load(deps.storage, msg.id)?
            .ok_or(ContractError::UnknownReplyID {})?;

        // If contract_addr matches THIS contract, it is the proxy callback
        // proxy_callback is also responsible for handling reply modes: "handle_failure", "handle_success"
        // TODO: Replace by `contains()` if possible `https://github.com/rust-lang/rust/issues/62358`
        if queue_item
            .contract_addr
            .as_ref()
            .map_or(false, |addr| *addr == env.contract.address)
        {
            let task =
                self.task_after_action(deps.storage, deps.api, queue_item, msg.result.is_ok())?;
            let reply_submsg_failed = msg.result.failed();
            let queue_item = self.rq_update_rq_item(deps.storage, msg.id, reply_submsg_failed)?;
            if queue_item.action_idx == task.actions.len() as u64 {
                // Last action
                self.rq_remove(deps.storage, msg.id);
                return self.proxy_callback(deps, env, msg, task, queue_item);
            } else {
                return Ok(Response::new()
                    .add_attribute("reply", "processing_action")
                    .add_attribute("action_idx", queue_item.action_idx.to_string()));
            }
        }

        // NOTE: Currently only handling proxy callbacks
        // Responds with the reply ID if nothing was found in queue
        Ok(Response::new().add_attribute("reply_id", msg.id.to_string()))
    }
}
