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
// pub const PARTICIPANT2: &str = "cosmos1far5cqkvny7k9wq53aw0k42v3f76rcylzzv05n";
// pub const PARTICIPANT3: &str = "cosmos1xj3xagnprtqpfnvyp7k393kmes73rpuxqgamd8";
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
    factory_addr: &Addr,
    sender: Option<&str>,
    owner: Option<String>,
) -> (u64, Addr) {
    let code_id = app.store_code(contracts::croncat_manager_contract());
    let msg = croncat_manager::msg::InstantiateMsg {
        denom: DENOM.to_owned(),
        croncat_factory_addr: factory_addr.to_string(),
        croncat_tasks_key: ("tasks".to_owned(), [0, 1]),
        croncat_agents_key: ("agents".to_owned(), [0, 1]),
        owner_addr: Addr::unchecked(owner.unwrap_or(ADMIN.to_string())),
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
                contract_name: "manager".to_owned(),
            },
        )
        .unwrap();
    (code_id, metadata.unwrap().contract_addr)
}

pub(crate) fn init_manager(
    app: &mut App,
    sender: Option<&str>,
    owner: Option<String>,
    factory_addr: &Addr,
) -> Addr {
    let code_id = app.store_code(contracts::croncat_manager_contract());
    let msg = croncat_manager::msg::InstantiateMsg {
        denom: DENOM.to_owned(),
        croncat_factory_addr: factory_addr.to_string(),
        croncat_tasks_key: ("tasks".to_owned(), [0, 1]),
        croncat_agents_key: ("agents".to_owned(), [0, 1]),
        owner_addr: Addr::unchecked(owner.unwrap_or(ADMIN.to_string())),
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
                contract_name: "manager".to_owned(),
            },
        )
        .unwrap();
    metadata.unwrap().contract_addr
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
    msg: &InstantiateMsg,
    factory_addr: &Addr,
) -> Addr {
    let code_id = app.store_code(contracts::croncat_tasks_contract());
    let module_instantiate_info = ModuleInstantiateInfo {
        code_id,
        version: [0, 1],
        commit_id: "commit1".to_owned(),
        checksum: "checksum2".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(msg).unwrap(),
        contract_name: "tasks".to_owned(),
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
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
    metadata.unwrap().contract_addr
}

pub(crate) fn init_contracts(app: &mut App) -> (u64, Addr, Addr, Addr) {
    let (_, factory_addr) = init_croncat_factory(app);
    let (_, croncat_manager_addr) =
        init_croncat_manager_contract(app,factory_addr.as_str(),None, None);
    let (_, croncat_tasks_addr) =
        init_croncat_tasks_contract(app, None,);
    let (code_id, contract_addr) = init_agents_contract(
        app,
        None,
        None,
        factory_addr.as_str().clone(),
        croncat_manager_addr.as_str().clone(),
        croncat_tasks_addr.as_str().clone(),
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
    factory_addr: &str,
    manager_addr: &str,
    tasks_addr: &str,
) -> (u64, Addr) {
    let code_id = app.store_code(contracts::croncat_agents_contract());
    let msg = croncat_agents::msg::InstantiateMsg {
        manager_addr,
        owner_addr: owner.unwrap_or(ADMIN.to_string()),
        tasks_addr,
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
        sender.unwrap_or(Addr::unchecked(ADMIN)),
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
                contract_name: "agents".to_owned(),
            },
        )
        .unwrap();
    (code_id, metadata.unwrap().contract_addr)
}

//Factory
pub(crate) fn init_croncat_factory(app: &mut App) -> (u64, Addr) {
    let code_id = app.store_code(contracts::croncat_factory_contract());
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
