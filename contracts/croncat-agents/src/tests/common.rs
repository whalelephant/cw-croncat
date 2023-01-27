
use crate::msg::*;
use crate::state::{
    DEFAULT_MIN_COINS_FOR_AGENT_REGISTRATION, DEFAULT_MIN_TASKS_PER_AGENT,
    DEFAULT_NOMINATION_DURATION,
};
use cosmwasm_std::{coins, Addr, Empty};
use cosmwasm_std::{BlockInfo, Coin};
use croncat_sdk_agents::types::Config;
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

pub const AGENT0: &str = "agent0a7uhnpqthunr2rzj0ww0hwurpn42wyun6c5puz";
pub const AGENT1: &str = "agent17muvdgkep4ndptnyg38eufxsssq8jr3wnkysy8";
pub const AGENT2: &str = "agent2qxywje86amll9ptzxmla5ah52uvsd9f7drs2dl";
pub const AGENT3: &str = "agent3c3cy3wzzz3698ypklvh7shksvmefj69xhm89z2";
pub const AGENT4: &str = "agent4ykfcyj8fl6xzs88tsls05x93gmq68a7km05m4j";
pub const AGENT5: &str = "agent5k5k7y4hgy5lkq0kj3k3e9k38lquh0m66kxsu5c";
pub const AGENT6: &str = "agent614a8clxc49z9e3mjzhamhkprt2hgf0y53zczzj0";

pub const AGENT_BENEFICIARY: &str = "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
pub const ADMIN: &str = "cosmos1sjllsnramtg3ewxqwwrwjxfgc4n4ef9u0tvx7u";

pub const ANYONE: &str = "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
pub const PARTICIPANT0: &str = "cosmos1055rfv3fv0zxsp8h3x88mctnm7x9mlgmf4m4d6";
pub const PARTICIPANT1: &str = "cosmos1c3cy3wzzz3698ypklvh7shksvmefj69xhm89z2";
pub const PARTICIPANT2: &str = "cosmos1far5cqkvny7k9wq53aw0k42v3f76rcylzzv05n";
pub const PARTICIPANT3: &str = "cosmos1xj3xagnprtqpfnvyp7k393kmes73rpuxqgamd8";
pub const NATIVE_DENOM: &str = "uatom";

pub(crate) fn mock_config(manager_addr: &str, tasks_addr: &str) -> Config {
    Config {
        paused: false,
        owner_addr: Addr::unchecked(ADMIN),
        min_tasks_per_agent: DEFAULT_MIN_TASKS_PER_AGENT,
        agent_nomination_duration: DEFAULT_NOMINATION_DURATION,
        manager_addr: Addr::unchecked(manager_addr),
        min_coins_for_agent_registration: DEFAULT_MIN_COINS_FOR_AGENT_REGISTRATION,
        tasks_addr: Addr::unchecked(tasks_addr),
    }
}
pub(crate) fn mock_update_config(manager_addr: &str, tasks_addr: &str) -> UpdateConfig {
    UpdateConfig {
        owner_addr: Some(ADMIN.to_string()),
        paused: Some(false),
        min_tasks_per_agent: Some(DEFAULT_MIN_TASKS_PER_AGENT),
        agent_nomination_duration: Some(DEFAULT_NOMINATION_DURATION),
        manager_addr: Some(manager_addr.to_string()),
        min_coins_for_agent_registration: None,
        tasks_addr: Some(tasks_addr.to_string()),
    }
}

pub(crate) fn default_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        let accounts: Vec<(u128, String)> = vec![
            (6_000_000, ADMIN.to_string()),
            (500_000, ANYONE.to_string()),
            (500_000, AGENT0.to_string()),
            (500_000, AGENT1.to_string()),
            (500_000, AGENT2.to_string()),
            (500_000, AGENT3.to_string()),
            (500_000, AGENT4.to_string()),
            (500_000, AGENT5.to_string()),
            (500_000, AGENT6.to_string()),
        ];
        for (amt, address) in accounts {
            let coin = coins(amt, format!("{}", NATIVE_DENOM));
            router
                .bank
                .init_balance(storage, &Addr::unchecked(address), coin)
                .unwrap();
        }
    })
}

pub(crate) fn agent_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub(crate) fn croncat_manager_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        croncat_manager::contract::execute,
        croncat_manager::contract::instantiate,
        croncat_manager::contract::query,
    )
    .with_reply(croncat_manager::contract::reply);
    Box::new(contract)
}

pub(crate) fn default_manager_instantiate_message() -> croncat_manager::msg::InstantiateMsg {
    croncat_manager::msg::InstantiateMsg {
        denom: NATIVE_DENOM.to_owned(),
        croncat_factory_addr: "croncat_factory_addr".to_owned(),
        croncat_tasks_key: ("croncat_tasks_name".to_owned(), [0, 1]),
        croncat_agents_key: ("croncat_agents_name".to_owned(), [0, 1]),
        owner_addr: None,
        gas_price: None,
        treasury_addr: None,
    }
}
pub(crate) fn init_croncat_manager_contract(
    app: &mut App,
    sender: Option<&str>,
    owner: Option<String>,
    funds: Option<&[Coin]>,
) -> (u64, Addr) {
    let manager_code_id = app.store_code(croncat_manager_contract());
    let manager_contract_addr = app
        .instantiate_contract(
            manager_code_id,
            Addr::unchecked(sender.unwrap_or(ADMIN)),
            &default_manager_instantiate_message(),
            funds.unwrap_or(&[]),
            "manager",
            owner,
        )
        .unwrap();

    (manager_code_id, manager_contract_addr)
}

pub(crate) fn croncat_tasks_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        croncat_tasks::contract::execute,
        croncat_tasks::contract::instantiate,
        croncat_tasks::contract::query,
    );
    Box::new(contract)
}
pub(crate) fn default_croncat_tasks_instantiate_msg() -> croncat_tasks::msg::InstantiateMsg {
    croncat_tasks::msg::InstantiateMsg {
        chain_name: "atom".to_owned(),
        owner_addr: None,
        croncat_manager_key: ("manager".to_owned(), [0, 1]),
        croncat_agents_key: ("agents".to_owned(), [0, 1]),
        slot_granularity_time: None,
        gas_base_fee: None,
        gas_action_fee: None,
        gas_query_fee: None,
        gas_limit: None,
    }
}
pub(crate) fn init_croncat_tasks_contract(
    app: &mut App,
    sender: Option<&str>,
    owner: Option<String>,
    funds: Option<&[Coin]>,
) -> (u64, Addr) {
    let tasks_code_id = app.store_code(croncat_tasks_contract());
    let tasks_contract_addr = app
        .instantiate_contract(
            tasks_code_id,
            Addr::unchecked(sender.unwrap_or(ADMIN)),
            &default_croncat_tasks_instantiate_msg(),
            funds.unwrap_or(&[]),
            "manager",
            owner,
        )
        .unwrap();

    (tasks_code_id, tasks_contract_addr)
}
pub(crate) fn init_contracts(app: &mut App, sender: Option<&str>) -> (u64, Addr, Addr, Addr) {
    let (_, croncat_manager_addr) =
        init_croncat_manager_contract(app, sender, Some(ADMIN.to_string()), None);
    let (_, croncat_tasks_addr) =
        init_croncat_tasks_contract(app, sender, Some(ADMIN.to_string()), None);
    let (code_id, contract_addr) = init_agents_contract(
        app,
        None,
        None,
        croncat_manager_addr.clone(),
        croncat_tasks_addr.clone(),
        None,
        None,
    );
    (
        code_id,
        contract_addr,
        croncat_manager_addr,
        croncat_tasks_addr,
    )
}
pub(crate) fn init_agents_contract(
    app: &mut App,
    sender: Option<&str>,
    owner: Option<String>,
    croncat_manager_addr: Addr,
    croncat_tasks_addr: Addr,
    init_msg: Option<InstantiateMsg>,
    funds: Option<&[Coin]>,
) -> (u64, Addr) {
    let contract_code_id = app.store_code(agent_contract());
    let init_msg = init_msg.unwrap_or(InstantiateMsg {
        owner_addr: owner,
        agent_nomination_duration: None,
        min_tasks_per_agent: Some(2),
        manager_addr: croncat_manager_addr.to_string(),
        tasks_addr: croncat_tasks_addr.to_string(),
        min_coin_for_agent_registration: None,
    });
    let contract_addr = app
        .instantiate_contract(
            contract_code_id,
            Addr::unchecked(sender.unwrap_or(ADMIN)),
            &init_msg,
            funds.unwrap_or(&[]),
            "agents",
            None,
        )
        .unwrap();

    (contract_code_id, contract_addr)
}

pub(crate) fn add_seconds_to_block(block: &mut BlockInfo, seconds: u64) {
    block.time = block.time.plus_seconds(seconds);
}
pub(crate) fn increment_block_height(block: &mut BlockInfo, inc_value: Option<u64>) {
    block.height += inc_value.unwrap_or(1);
}
