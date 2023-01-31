use crate::msg::*;
use crate::state::{
    DEFAULT_MIN_COINS_FOR_AGENT_REGISTRATION, DEFAULT_MIN_TASKS_PER_AGENT,
    DEFAULT_NOMINATION_DURATION,
};
use cosmwasm_std::BlockInfo;
use cosmwasm_std::{coins, to_binary, Addr, Empty};
use croncat_sdk_factory::msg::{ContractMetadataResponse, ModuleInstantiateInfo, VersionKind};
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
pub const PARTICIPANT2: &str = "cosmos2far5cqkvny7k9wq53aw0k42v3f76rcylzzv05n";
pub const PARTICIPANT3: &str = "cosmos3xj3xagnprtqpfnvyp7k393kmes73rpuxqgamd8";
pub const PARTICIPANT4: &str = "cosmos4t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
pub const PARTICIPANT5: &str = "cosmos5t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
pub const PARTICIPANT6: &str = "cosmos6t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
pub const PARTICIPANT7: &str = "cosmos7t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";

pub const NATIVE_DENOM: &str = "uatom";

#[allow(dead_code)]
pub(crate) struct TestScope {
    pub croncat_factory_addr: Addr,
    pub croncat_agents_addr: Addr,
    pub croncat_agents_code_id: Option<u64>,
    pub croncat_manager_addr: Addr,
    pub croncat_tasks_addr: Addr,
}
pub(crate) fn mock_config(croncat_factory_addr: &str) -> Config {
    Config {
        paused: false,
        owner_addr: Addr::unchecked(ADMIN),
        min_tasks_per_agent: DEFAULT_MIN_TASKS_PER_AGENT,
        agent_nomination_duration: DEFAULT_NOMINATION_DURATION,
        croncat_factory_addr: Addr::unchecked(croncat_factory_addr.to_owned()),
        croncat_manager_key: ("manager".to_owned(), [4, 2]),
        croncat_tasks_key: ("tasks".to_owned(), [42, 0]),
        min_coins_for_agent_registration: DEFAULT_MIN_COINS_FOR_AGENT_REGISTRATION,
    }
}
pub(crate) fn mock_update_config(croncat_factory_addr: &str) -> UpdateConfig {
    UpdateConfig {
        owner_addr: Some(ADMIN.to_string()),
        paused: Some(false),
        min_tasks_per_agent: Some(DEFAULT_MIN_TASKS_PER_AGENT),
        agent_nomination_duration: Some(DEFAULT_NOMINATION_DURATION),
        croncat_factory_addr: Some(croncat_factory_addr.to_owned()),
        croncat_manager_key: Some(("manager".to_owned(), [4, 2])),
        croncat_tasks_key: Some(("tasks".to_owned(), [42, 0])),
        min_coins_for_agent_registration: None,
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
            let coin = coins(amt, NATIVE_DENOM);
            router
                .bank
                .init_balance(storage, &Addr::unchecked(address), coin)
                .unwrap();
        }
    })
}

pub(crate) fn croncat_agents_contract() -> Box<dyn Contract<Empty>> {
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

pub(crate) fn init_croncat_manager_contract(
    app: &mut App,
    sender: Option<&str>,
    owner: Option<String>,
    factory_addr: &str,
) -> (u64, Addr) {
    let code_id = app.store_code(croncat_manager_contract());
    let msg = croncat_manager::msg::InstantiateMsg {
        denom: NATIVE_DENOM.to_owned(),
        version: Some("0.1".to_owned()),
        croncat_tasks_key: ("tasks".to_owned(), [0, 1]),
        croncat_agents_key: ("agents".to_owned(), [0, 1]),
        owner_addr: Some(owner.unwrap_or_else(|| ADMIN.to_string())),
        gas_price: None,
        treasury_addr: None,
    };
    let module_instantiate_info = ModuleInstantiateInfo {
        code_id,
        version: [0, 1],
        commit_id: "commit1".to_owned(),
        checksum: "checksum2".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&msg).unwrap(),
        contract_name: "manager".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(sender.unwrap_or(ADMIN)),
        Addr::unchecked(factory_addr.to_owned()),
        &croncat_factory::msg::ExecuteMsg::Deploy {
            kind: VersionKind::Tasks,
            module_instantiate_info,
        },
        &[],
    )
    .unwrap();

    let metadata: Option<ContractMetadataResponse> = app
        .wrap()
        .query_wasm_smart(
            factory_addr,
            &croncat_factory::msg::QueryMsg::LatestContract {
                contract_name: "manager".to_owned(),
            },
        )
        .unwrap();
    (code_id, metadata.unwrap().contract_addr)
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
        version: Some("0.1".to_owned()),
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
    _: Option<String>,
    factory_addr: &Addr,
) -> (u64, Addr) {
    let code_id = app.store_code(croncat_tasks_contract());
    let module_instantiate_info = ModuleInstantiateInfo {
        code_id,
        version: [0, 1],
        commit_id: "commit1".to_owned(),
        checksum: "checksum2".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&default_croncat_tasks_instantiate_msg()).unwrap(),
        contract_name: "tasks".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(sender.unwrap_or(ADMIN)),
        factory_addr.to_owned(),
        &croncat_factory::msg::ExecuteMsg::Deploy {
            kind: VersionKind::Tasks,
            module_instantiate_info,
        },
        &[],
    )
    .unwrap();

    let metadata: Option<ContractMetadataResponse> = app
        .wrap()
        .query_wasm_smart(
            factory_addr,
            &croncat_factory::msg::QueryMsg::LatestContract {
                contract_name: "tasks".to_owned(),
            },
        )
        .unwrap();
    (code_id, metadata.unwrap().contract_addr)
}

pub(crate) fn init_test_scope(app: &mut App) -> TestScope {
    let (_, croncat_factory_addr) = init_croncat_factory(app);
    let (_, croncat_manager_addr) =
        init_croncat_manager_contract(app, None, None, croncat_factory_addr.as_str());
    let (_, croncat_tasks_addr) =
        init_croncat_tasks_contract(app, None, None, &croncat_factory_addr);
    let (croncat_agents_code_id, croncat_agents_addr) =
        init_agents_contract(app, None, None, croncat_factory_addr.as_str());

    TestScope {
        croncat_factory_addr,
        croncat_agents_code_id: Some(croncat_agents_code_id),
        croncat_agents_addr,
        croncat_manager_addr,
        croncat_tasks_addr,
    }
}

pub(crate) fn init_agents_contract(
    app: &mut App,
    sender: Option<&str>,
    owner: Option<String>,
    factory_addr: &str,
) -> (u64, Addr) {
    let code_id = app.store_code(croncat_agents_contract());
    let msg = InstantiateMsg {
        version: Some("0.1".to_owned()),
        croncat_manager_key: ("manager".to_owned(), [0, 1]),
        croncat_tasks_key: ("tasks".to_owned(), [0, 1]),
        owner_addr: Some(owner.unwrap_or_else(|| ADMIN.to_string())),
        agent_nomination_duration: None,
        min_tasks_per_agent: None,
        min_coin_for_agent_registration: None,
    };
    let module_instantiate_info = ModuleInstantiateInfo {
        code_id,
        version: [0, 1],
        commit_id: "commit1".to_owned(),
        checksum: "checksum2".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&msg).unwrap(),
        contract_name: "agents".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(sender.unwrap_or(ADMIN)),
        Addr::unchecked(factory_addr.to_owned()),
        &croncat_factory::msg::ExecuteMsg::Deploy {
            kind: VersionKind::Tasks,
            module_instantiate_info,
        },
        &[],
    )
    .unwrap();

    let metadata: Option<ContractMetadataResponse> = app
        .wrap()
        .query_wasm_smart(
            factory_addr,
            &croncat_factory::msg::QueryMsg::LatestContract {
                contract_name: "agents".to_owned(),
            },
        )
        .unwrap();
    (code_id, metadata.unwrap().contract_addr)
}

//Factory
pub(crate) fn init_croncat_factory(app: &mut App) -> (u64, Addr) {
    let code_id = app.store_code(croncat_factory_contract());
    let addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(ADMIN),
            &croncat_factory::msg::InstantiateMsg { owner_addr: None },
            &[],
            "croncat_factory",
            None,
        )
        .unwrap();
    (code_id, addr)
}
pub(crate) fn croncat_factory_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        croncat_factory::contract::execute,
        croncat_factory::contract::instantiate,
        croncat_factory::contract::query,
    )
    .with_reply(croncat_factory::contract::reply);
    Box::new(contract)
}

pub(crate) fn add_seconds_to_block(block: &mut BlockInfo, seconds: u64) {
    block.time = block.time.plus_seconds(seconds);
}
pub(crate) fn increment_block_height(block: &mut BlockInfo, inc_value: Option<u64>) {
    block.height += inc_value.unwrap_or(1);
}
