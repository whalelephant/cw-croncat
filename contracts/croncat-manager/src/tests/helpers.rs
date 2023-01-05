use cosmwasm_std::{coins, Addr, Coin};
use croncat_sdk_core::types::{BalancesResponse, Config};
use cw_multi_test::{App, AppBuilder, Executor};

use anyhow::Result as AnyResult;

use crate::msg::{InstantiateMsg, QueryMsg};

use super::{
    contracts, ADMIN, AGENT0, AGENT1, AGENT2, AGENT3, AGENT4, AGENT_BENEFICIARY, ANYONE, DENOM,
    PARTICIPANT0, PARTICIPANT1, PARTICIPANT2, PARTICIPANT3, PARTICIPANT4, PARTICIPANT5,
    PARTICIPANT6, VERY_RICH,
};

pub(crate) fn default_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        let accounts: Vec<(u128, String)> = vec![
            (6_000_000, ADMIN.to_string()),
            (500_000, ANYONE.to_string()),
            (2_000_000, AGENT0.to_string()),
            (2_000_000, AGENT1.to_string()),
            (2_000_000, AGENT2.to_string()),
            (2_000_000, AGENT3.to_string()),
            (2_000_000, AGENT4.to_string()),
            (500_0000, PARTICIPANT0.to_string()),
            (500_0000, PARTICIPANT1.to_string()),
            (500_0000, PARTICIPANT2.to_string()),
            (500_0000, PARTICIPANT3.to_string()),
            (500_0000, PARTICIPANT4.to_string()),
            (500_0000, PARTICIPANT5.to_string()),
            (500_0000, PARTICIPANT6.to_string()),
            (2_000_000, AGENT_BENEFICIARY.to_string()),
            (u128::max_value(), VERY_RICH.to_string()),
        ];
        for (amt, address) in accounts {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(address), coins(amt, DENOM))
                .unwrap();
        }
    })
}

pub(crate) fn init(app: &mut App, msg: InstantiateMsg, funds: &[Coin]) -> AnyResult<Addr> {
    let code_id = app.store_code(contracts::croncat_manager_contract());
    let addr = app.instantiate_contract(
        code_id,
        Addr::unchecked(ADMIN),
        &msg,
        funds,
        "croncat-manager",
        None,
    )?;
    Ok(addr)
}

pub(crate) fn default_instantiate_message() -> InstantiateMsg {
    InstantiateMsg {
        denom: DENOM.to_owned(),
        cw_rules_addr: "cw_rules_addr".to_owned(),
        croncat_tasks_addr: "croncat_tasks_addr".to_owned(),
        croncat_agents_addr: "croncat_agents_addr".to_owned(),
        owner_id: None,
        gas_base_fee: None,
        gas_action_fee: None,
        gas_query_fee: None,
        gas_wasm_query_fee: None,
        gas_price: None,
        agent_nomination_duration: None,
    }
}

pub(crate) fn query_manager_config(app: &App, manager: &Addr) -> Config {
    app.wrap()
        .query_wasm_smart(manager, &QueryMsg::Config {})
        .unwrap()
}

pub(crate) fn query_manager_balances(app: &App, manager: &Addr) -> BalancesResponse {
    app.wrap()
        .query_wasm_smart(
            manager,
            &QueryMsg::AvailableBalances {
                from_index: None,
                limit: None,
            },
        )
        .unwrap()
}
