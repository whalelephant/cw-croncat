use crate::error::ContractError;
use crate::helpers::GenericBalance;
use crate::state::{Config, CwCroncat};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;
use cw_croncat_core::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cw_croncat_core::traits::{BalancesOperations, ResultFailed};
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
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        // keep tally of balances initialized
        let mut native = deps.querier.query_all_balances(&env.contract.address)?;
        native.checked_add_coins(&info.funds)?;
        let available_balance = GenericBalance {
            native,
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
            cw_rules_addr: deps.api.addr_validate(&msg.cw_rules_addr)?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::test_helpers::mock_init;
    use crate::state::QueueItem;
    use cosmwasm_std::testing::{
        mock_dependencies_with_balance, mock_env, mock_info, MOCK_CONTRACT_ADDR,
    };
    use cosmwasm_std::{
        coin, coins, from_binary, Addr, Binary, Event, Reply, SubMsgResponse, SubMsgResult,
    };
    use cw_croncat_core::msg::{GetConfigResponse, QueryMsg};
    use cw_croncat_core::types::SlotType;
    const AGENT0: &str = "cosmos1a7uhnpqthunr2rzj0ww0hwurpn42wyun6c5puz";
    #[test]
    fn configure() {
        let mut deps = mock_dependencies_with_balance(&coins(200, ""));
        let mut store = CwCroncat::default();

        let msg = InstantiateMsg {
            denom: "atom".to_string(),
            owner_id: None,
            gas_base_fee: None,
            agent_nomination_duration: Some(360),
            cw_rules_addr: "todo".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "meow"));

        // we can just call .unwrap() to assert this was a success
        let res = store
            .instantiate(deps.as_mut(), mock_env(), info.clone(), msg)
            .unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = store
            .query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {})
            .unwrap();
        let value: GetConfigResponse = from_binary(&res).unwrap();
        assert_eq!(false, value.paused);
        assert_eq!(info.sender, value.owner_id);
        // assert_eq!(None, value.treasury_id);
        assert_eq!(3, value.min_tasks_per_agent);
        assert_eq!(
            vec![(SlotType::Block, 0, 0), (SlotType::Cron, 0, 0)],
            value.agent_active_indices
        );
        assert_eq!(600, value.agents_eject_threshold);
        assert_eq!("atom", value.native_denom);
        assert_eq!(coin(5, "atom"), value.agent_fee);
        assert_eq!(1, value.gas_price);
        assert_eq!(3, value.proxy_callback_gas);
        assert_eq!(60_000_000_000, value.slot_granularity);
    }

    #[test]
    fn replies() {
        let mut deps = mock_dependencies_with_balance(&coins(200, ""));
        let store = CwCroncat::default();
        mock_init(&store, deps.as_mut()).unwrap();
        let task_hash = "ad15b0f15010d57a51ff889d3400fe8d083a0dab2acfc752c5eb55e9e6281705"
            .as_bytes()
            .to_vec();
        let response = SubMsgResponse {
            data: Some(Binary::from_base64("MTMzNw==").unwrap()),
            events: vec![Event::new("wasm").add_attribute("cat", "meow")],
        };

        let mut msg = Reply {
            id: 1,
            result: SubMsgResult::Ok(response),
        };

        // Check there wasn't any known reply
        let res_err1 = store
            .reply(deps.as_mut(), mock_env(), msg.clone())
            .unwrap_err();
        assert_eq!(ContractError::UnknownReplyID {}, res_err1);

        // Create fake Queue item, check that it gets removed, returns default reply_id
        // TODO: Dont think it's possible to create fake queue items
        // store
        //     .rq_push(
        //         deps.as_mut().storage,
        //         QueueItem {
        //             action_idx: 0,
        //             task_hash: Some(task_hash.clone()),
        //             contract_addr: None,
        //             task_is_extra: Some(false),
        //             agent_id: Some(Addr::unchecked(AGENT0)),
        //             failed: false,
        //         },
        //     )
        //     .unwrap();
        // let queue_item1 = store
        //     .reply_queue
        //     .may_load(deps.as_mut().storage, msg.id)
        //     .unwrap();
        // assert!(queue_item1.is_some());

        // let res1 = store.reply(deps.as_mut(), mock_env(), msg.clone()).unwrap();
        // let mut has_reply_id: bool = false;
        // for a in res1.attributes {
        //     if a.key == "reply_id" && a.value == "1" {
        //         has_reply_id = true;
        //     }
        // }
        // assert!(has_reply_id);
        // let queue_item2 = store
        //     .reply_queue
        //     .may_load(deps.as_mut().storage, msg.id)
        //     .unwrap();
        // assert!(queue_item2.is_none());

        // Create fake Queue item with known contract address,
        // check that it gets removed, the rest is covered in proxy_callback tests
        store
            .rq_push(
                deps.as_mut().storage,
                QueueItem {
                    action_idx: 0,
                    task_hash: Some(task_hash),
                    contract_addr: Some(Addr::unchecked(MOCK_CONTRACT_ADDR)),
                    task_is_extra: Some(false),
                    agent_id: Some(Addr::unchecked(AGENT0)),
                    failed: false,
                },
            )
            .unwrap();
        msg.id = 1;
        let queue_item3 = store
            .reply_queue
            .may_load(deps.as_mut().storage, msg.id)
            .unwrap();
        assert!(queue_item3.is_some());

        let res_err2 = store
            .reply(deps.as_mut(), mock_env(), msg.clone())
            .unwrap_err();
        assert_eq!(ContractError::NoTaskFound {}, res_err2);
        // It can't get removed, because contract will rollback to original state at failure
        // TODO: retest it with integration tests
        // let queue_item4 = store
        //     .reply_queue
        //     .may_load(deps.as_mut().storage, msg.id)
        //     .unwrap();
        // assert!(queue_item4.is_some());
    }
}
