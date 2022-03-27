use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg};
use crate::state::{Config, CONFIG};
use cosmwasm_std::{Addr, Deps, DepsMut, MessageInfo, Response, StdResult};

pub(crate) fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let c: Config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        paused: c.paused,
        owner_id: c.owner_id,
        treasury_id: c.treasury_id,
        agent_task_ratio: c.agent_task_ratio,
        agent_active_index: c.agent_active_index,
        agents_eject_threshold: c.agents_eject_threshold,
        native_denom: c.native_denom,
        agent_fee: c.agent_fee,
        gas_price: c.gas_price,
        proxy_callback_gas: c.proxy_callback_gas,
        slot_granularity: c.slot_granularity,
    })
}

/// Changes core configurations
/// Should only be updated by owner -- in best case DAO based :)
pub fn update_settings(
    deps: DepsMut,
    info: MessageInfo,
    payload: ExecuteMsg,
) -> Result<Response, ContractError> {
    match payload {
        ExecuteMsg::UpdateSettings {
            owner_id,
            slot_granularity,
            paused,
            agent_fee,
            gas_price,
            proxy_callback_gas,
            agent_task_ratio,
            agents_eject_threshold,
            treasury_id,
        } => {
            CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
                if info.sender != config.owner_id {
                    return Err(ContractError::Unauthorized {});
                }

                if let Some(owner_id) = owner_id {
                    config.owner_id = owner_id;
                }
                if let Some(treasury_id) = treasury_id {
                    config.treasury_id = Some(treasury_id);
                }

                if let Some(slot_granularity) = slot_granularity {
                    config.slot_granularity = slot_granularity;
                }
                if let Some(paused) = paused {
                    config.paused = paused;
                }
                if let Some(gas_price) = gas_price {
                    config.gas_price = gas_price;
                }
                if let Some(proxy_callback_gas) = proxy_callback_gas {
                    config.proxy_callback_gas = proxy_callback_gas;
                }
                if let Some(agent_fee) = agent_fee {
                    config.agent_fee = agent_fee;
                }
                if let Some(agent_task_ratio) = agent_task_ratio {
                    config.agent_task_ratio = [agent_task_ratio[0], agent_task_ratio[1]];
                }
                if let Some(agents_eject_threshold) = agents_eject_threshold {
                    config.agents_eject_threshold = agents_eject_threshold;
                }
                Ok(config)
            })?;
        }
        _ => unreachable!(),
    }
    let c: Config = CONFIG.load(deps.storage)?;
    Ok(Response::new()
        .add_attribute("method", "update_settings")
        .add_attribute("paused", c.paused.to_string())
        .add_attribute("owner_id", c.owner_id.to_string())
        .add_attribute(
            "treasury_id",
            c.treasury_id.unwrap_or(Addr::unchecked("")).to_string(),
        )
        .add_attribute(
            "agent_task_ratio",
            c.agent_task_ratio
                .to_vec()
                .into_iter()
                .map(|i| i.to_string())
                .collect::<String>(),
        )
        .add_attribute("agent_active_index", c.agent_active_index.to_string())
        .add_attribute(
            "agents_eject_threshold",
            c.agents_eject_threshold.to_string(),
        )
        .add_attribute("native_denom", c.native_denom)
        .add_attribute("agent_fee", c.agent_fee.to_string())
        .add_attribute("gas_price", c.gas_price.to_string())
        .add_attribute("proxy_callback_gas", c.proxy_callback_gas.to_string())
        .add_attribute("slot_granularity", c.slot_granularity.to_string()))
}

// /// Allows admin to calculate internal balances
// /// Returns surplus and rewards balances
// /// Can be used to measure how much surplus is remaining for staking / etc
// #[private]
// pub fn calc_balances(&mut self) -> (U128, U128) {
//     let base_balance = BASE_BALANCE; // safety overhead
//     let storage_balance = env::storage_byte_cost().saturating_mul(env::storage_usage() as u128);

//     // Using storage + threshold as the start for how much balance is required
//     let required_balance = base_balance.saturating_add(storage_balance);
//     let mut total_task_balance: Balance = 0;
//     let mut total_reward_balance: Balance = 0;

//     // Loop all tasks and add
//     for (_, t) in self.tasks.iter() {
//         total_task_balance = total_task_balance.saturating_add(t.total_deposit.0);
//     }

//     // Loop all agents rewards and add
//     for a in self.agent_active_queue.iter() {
//         if let Some(agent) = self.agents.get(&a) {
//             total_reward_balance = total_reward_balance.saturating_add(agent.balance.0);
//         }
//     }

//     let total_available_balance: Balance =
//         total_task_balance.saturating_add(total_reward_balance);

//     // Calculate surplus, which could be used for staking
//     // TODO: This would be adjusted by preferences of like 30% of total task deposit or similar
//     let surplus = u128::max(
//         env::account_balance()
//             .saturating_sub(total_available_balance)
//             .saturating_sub(required_balance),
//         0,
//     );
//     log!("Stakeable surplus {}", surplus);

//     // update internal values
//     self.available_balance = u128::max(total_available_balance, 0);

//     // Return surplus value in case we want to trigger staking based off outcome
//     (U128::from(surplus), U128::from(total_reward_balance))
// }

// /// Move Balance
// /// Allows owner to move balance to DAO or to let treasury transfer to itself only.
// pub fn move_balance(&mut self, amount: U128, account_id: AccountId) -> Promise {
//     // Check if is owner OR the treasury account
//     let transfer_warning = b"Not approved for transfer";
//     if let Some(treasury_id) = self.treasury_id.clone() {
//         if treasury_id != env::predecessor_account_id()
//             && self.owner_id != env::predecessor_account_id()
//         {
//             env::panic(transfer_warning);
//         }
//     } else if self.owner_id != env::predecessor_account_id() {
//         env::panic(transfer_warning);
//     }
//     // for now, only allow movement of funds between owner and treasury
//     let check_account = self.treasury_id.clone().unwrap_or(self.owner_id.clone());
//     if check_account != account_id.clone() {
//         env::panic(b"Cannot move funds to this account");
//     }
//     // Check that the amount is not larger than available
//     let (_, _, _, surplus) = self.get_balances();
//     assert!(amount.0 < surplus.0, "Amount is too high");

//     // transfer
//     // NOTE: Not updating available balance, as we are simply allowing surplus transfer only
//     Promise::new(account_id).transfer(amount.0)
// }

// /// Allows admin to remove slot data, in case a task gets stuck due to missed exits
// pub fn remove_slot_owner(&mut self, slot: U128) {
//     // assert_eq!(
//     //     self.owner_id,
//     //     env::predecessor_account_id(),
//     //     "Must be owner"
//     // );
//     assert_eq!(
//         env::current_account_id(),
//         env::predecessor_account_id(),
//         "Must be owner"
//     );
//     self.slots.remove(&slot.0);
// }

// /// Deletes a task in its entirety, returning any remaining balance to task owner.
// ///
// /// ```bash
// /// near call manager_v1.croncat.testnet remove_task_owner '{"task_hash": ""}' --accountId YOU.testnet
// /// ```
// #[private]
// pub fn remove_task_owner(&mut self, task_hash: Base64VecU8) {
//     let hash = task_hash.0;
//     self.tasks.get(&hash).expect("No task found by hash");

//     // If owner, allow to remove task
//     self.exit_task(hash);
// }

// /// Deletes a trigger in its entirety, only by owner.
// ///
// /// ```bash
// /// near call manager_v1.croncat.testnet remove_trigger_owner '{"trigger_hash": ""}' --accountId YOU.testnet
// /// ```
// #[private]
// pub fn remove_trigger_owner(&mut self, trigger_hash: Base64VecU8) {
//     self.triggers
//         .remove(&trigger_hash.0)
//         .expect("No trigger found by hash");
// }
// }

#[cfg(test)]
mod tests {
    // use super::*;
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn test_update_settings() {
        let mut deps = mock_dependencies_with_balance(&coins(200, ""));

        let msg = InstantiateMsg { owner_id: None };
        let info = mock_info("creator", &coins(1000, "meow"));
        let res_init = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res_init.messages.len());

        // do the thing
        let payload = ExecuteMsg::UpdateSettings {
            paused: Some(true),
            owner_id: None,
            treasury_id: None,
            agent_fee: None,
            agent_task_ratio: None,
            agents_eject_threshold: None,
            gas_price: None,
            proxy_callback_gas: None,
            slot_granularity: None,
        };
        let res_exec = execute(deps.as_mut(), mock_env(), info.clone(), payload).unwrap();
        assert_eq!(0, res_exec.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap();
        let value: ConfigResponse = from_binary(&res).unwrap();
        println!("CONFIG {:?}", value);
        assert_eq!(true, value.paused);
        assert_eq!(info.sender, value.owner_id);
    }

    // #[test]
    // fn increment() {
    //     let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Increment {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(18, value.count);
    // }

    // #[test]
    // fn reset() {
    //     let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // only the original creator can reset the counter
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    //     // should now be 5
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);
    // }
}
