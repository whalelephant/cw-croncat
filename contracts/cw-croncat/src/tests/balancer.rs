use crate::balancer::{Balancer, BalancerMode, RoundRobinBalancer};
use crate::contract::{
    GAS_ACTION_FEE, GAS_ADJUSTMENT_NUMERATOR_DEFAULT, GAS_BASE_FEE, GAS_DENOMINATOR,
    GAS_NUMERATOR_DEFAULT, GAS_QUERY_FEE, GAS_WASM_QUERY_FEE,
};
use crate::state::{Config, TaskInfo};
use crate::tests::helpers::{default_task, AGENT0, AGENT1, AGENT2, AGENT3, AGENT4, AGENT5};
use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env};
use cosmwasm_std::{coins, Addr};
use cw_croncat_core::types::{GasPrice, GenericBalance, SlotType};

use crate::CwCroncat;

use super::helpers::{ADMIN, NATIVE_DENOM};

fn mock_config() -> Config {
    Config {
        paused: false,
        owner_id: Addr::unchecked(ADMIN),
        chain_name: "atom".to_string(),
        // treasury_id: None,
        min_tasks_per_agent: 3,
        agent_active_indices: Vec::<(SlotType, u32, u32)>::with_capacity(0),
        agents_eject_threshold: 600, // how many slots an agent can miss before being ejected. 10 * 60 = 1hr
        available_balance: GenericBalance::default(),
        staked_balance: GenericBalance::default(),
        agent_fee: 5,
        gas_price: GasPrice {
            numerator: GAS_NUMERATOR_DEFAULT,
            denominator: GAS_DENOMINATOR,
            gas_adjustment_numerator: GAS_ADJUSTMENT_NUMERATOR_DEFAULT,
        },
        gas_action_fee: GAS_ACTION_FEE,
        gas_query_fee: GAS_QUERY_FEE,
        gas_wasm_query_fee: GAS_WASM_QUERY_FEE,
        proxy_callback_gas: 3,
        slot_granularity_time: 60_000_000_000,
        native_denom: NATIVE_DENOM.to_owned(),
        cw20_whitelist: vec![],
        agent_nomination_duration: 9,
        limit: 100,
        cw_rules_addr: Addr::unchecked("todo"),
        gas_base_fee: GAS_BASE_FEE,
    }
}
#[test]
fn test_agent_has_valid_task_count_ao_mode() {
    let store = CwCroncat::default();
    let mut deps = mock_dependencies_with_balance(&coins(200, NATIVE_DENOM));
    let env = mock_env();
    let mut balancer = RoundRobinBalancer::default();
    let config = mock_config();

    store.config.save(&mut deps.storage, &config).unwrap();

    let mut active_agents: Vec<Addr> = store
        .agent_active_queue
        .may_load(&deps.storage)
        .unwrap()
        .unwrap_or_default();
    active_agents.extend(vec![
        Addr::unchecked(AGENT0),
        Addr::unchecked(AGENT1),
        Addr::unchecked(AGENT2),
        Addr::unchecked(AGENT3),
        Addr::unchecked(AGENT4),
    ]);

    store
        .agent_active_queue
        .save(&mut deps.storage, &active_agents)
        .unwrap();
    let slot: (Option<u64>, Option<u64>) = (Some(1), Some(2));
    let result = balancer
        .get_agent_tasks(
            &deps.as_ref(),
            &env.clone(),
            &store.config,
            &store.agent_active_queue,
            Addr::unchecked(AGENT0),
            slot,
        )
        .unwrap()
        .unwrap();
    assert_eq!(result.num_block_tasks.u64(), 1);
    assert_eq!(result.num_cron_tasks.u64(), 1);

    //Verify earch gents valid amount
    let slot: (Option<u64>, Option<u64>) = (Some(100), Some(50));
    let result = balancer
        .get_agent_tasks(
            &deps.as_ref(),
            &env.clone(),
            &store.config,
            &store.agent_active_queue,
            Addr::unchecked(AGENT0),
            slot,
        )
        .unwrap()
        .unwrap();
    assert!(result.num_block_tasks.u64() == 20);
    assert!(result.num_cron_tasks.u64() == 10);

    //Verify agents gets zero
    let slot: (Option<u64>, Option<u64>) = (Some(0), Some(0));
    let result = balancer
        .get_agent_tasks(
            &deps.as_ref(),
            &env.clone(),
            &store.config,
            &store.agent_active_queue,
            Addr::unchecked(AGENT0),
            slot,
        )
        .unwrap()
        .unwrap();
    assert!(result.num_block_tasks.u64() == 0);
    assert!(result.num_cron_tasks.u64() == 0);
}

#[test]
fn test_check_valid_agents_get_extra_tasks_ao_mode() {
    let store = CwCroncat::default();
    let mut deps = mock_dependencies_with_balance(&coins(200, NATIVE_DENOM));
    let env = mock_env();
    let mut balancer = RoundRobinBalancer::default();
    let config = mock_config();

    store.config.save(&mut deps.storage, &config).unwrap();

    let mut active_agents: Vec<Addr> = store
        .agent_active_queue
        .may_load(&deps.storage)
        .unwrap()
        .unwrap_or_default();
    active_agents.extend(vec![
        Addr::unchecked(AGENT0),
        Addr::unchecked(AGENT1),
        Addr::unchecked(AGENT2),
        Addr::unchecked(AGENT3),
        Addr::unchecked(AGENT4),
    ]);

    store
        .agent_active_queue
        .save(&mut deps.storage, &active_agents)
        .unwrap();

    //Verify agent0 gets extra
    let slot: (Option<u64>, Option<u64>) = (Some(7), Some(7));
    let result = balancer
        .get_agent_tasks(
            &deps.as_ref(),
            &env.clone(),
            &store.config,
            &store.agent_active_queue,
            Addr::unchecked(AGENT0),
            slot,
        )
        .unwrap()
        .unwrap();

    assert_eq!(result.num_block_tasks.u64(), 2);
    assert_eq!(result.num_cron_tasks.u64(), 2);
    assert_eq!(result.num_block_tasks_extra.u64(), 1);
    assert_eq!(result.num_cron_tasks_extra.u64(), 1);

    //Verify agent1 gets extra
    let result = balancer
        .get_agent_tasks(
            &deps.as_ref(),
            &env.clone(),
            &store.config,
            &store.agent_active_queue,
            Addr::unchecked(AGENT1),
            slot,
        )
        .unwrap()
        .unwrap();

    assert_eq!(result.num_block_tasks.u64(), 2);
    assert_eq!(result.num_cron_tasks.u64(), 2);
    assert_eq!(result.num_block_tasks_extra.u64(), 1);
    assert_eq!(result.num_cron_tasks_extra.u64(), 1);

    //Verify agent3 not getting extra
    let result = balancer
        .get_agent_tasks(
            &deps.as_ref(),
            &env.clone(),
            &store.config,
            &store.agent_active_queue,
            Addr::unchecked(AGENT3),
            slot,
        )
        .unwrap()
        .unwrap();

    assert_eq!(result.num_block_tasks.u64(), 1);
    assert_eq!(result.num_cron_tasks.u64(), 1);
    assert_eq!(result.num_block_tasks_extra.u64(), 0);
    assert_eq!(result.num_cron_tasks_extra.u64(), 0);
}

//EQ Mode
#[test]
fn test_check_valid_agents_get_tasks_eq_mode() {
    let store = CwCroncat::default();
    let mut deps = mock_dependencies_with_balance(&coins(200, NATIVE_DENOM));
    let env = mock_env();
    let mut balancer = RoundRobinBalancer::new(BalancerMode::Equalizer);
    let mut config = mock_config();
    store.config.save(&mut deps.storage, &config).unwrap();

    let mut local_test = |slots: (Option<u64>, Option<u64>),
                          act_agents: &[(&str, u64, u64)],
                          expected: &[(&str, u64, u64)]| {
        config.agent_active_indices = Vec::with_capacity(0);
        store.config.save(&mut deps.storage, &config).unwrap();

        store.agent_active_queue.remove(&mut deps.storage);
        let mut active_agents: Vec<Addr> = store
            .agent_active_queue
            .may_load(&deps.storage)
            .unwrap()
            .unwrap_or_default();
        active_agents.extend(act_agents.iter().map(|mapped| Addr::unchecked(mapped.0)));
        store
            .agent_active_queue
            .save(&mut deps.storage, &active_agents)
            .unwrap();

        let mut result = Vec::<(&str, u64, u64)>::new();
        act_agents.iter().for_each(|f| {
            if f.1 > 0 {
                let task_info = TaskInfo {
                    task: default_task(),
                    task_hash: "".as_bytes().to_vec(),
                    task_is_extra: Some(true),
                    agent_id: Addr::unchecked(f.0),
                    slot_kind: SlotType::Block,
                };
                balancer
                    .on_task_completed(
                        &mut deps.storage,
                        &env,
                        &store.config,
                        &store.agent_active_queue,
                        &task_info,
                    )
                    .unwrap();
            }
            if f.1 > 0 {
                let task_info = TaskInfo {
                    task: default_task(),
                    task_hash: "".as_bytes().to_vec(),
                    task_is_extra: Some(true),
                    agent_id: Addr::unchecked(f.0),
                    slot_kind: SlotType::Cron,
                };
                balancer
                    .on_task_completed(
                        &mut deps.storage,
                        &env,
                        &store.config,
                        &store.agent_active_queue,
                        &task_info,
                    )
                    .unwrap();
            }
        });
        for a in act_agents {
            let balancer_result = balancer
                .get_agent_tasks(
                    &deps.as_ref(),
                    &env.clone(),
                    &store.config,
                    &store.agent_active_queue,
                    Addr::unchecked(a.0),
                    slots,
                )
                .unwrap()
                .unwrap();
            result.push((
                a.0,
                balancer_result.num_block_tasks.u64(),
                balancer_result.num_cron_tasks.u64(),
            ));
        }
        assert_eq!(expected, &result);
    };

    local_test(
        (Some(7), Some(7)),
        &[
            (AGENT0, 0, 0),
            (AGENT1, 0, 0),
            (AGENT2, 0, 0),
            (AGENT3, 0, 0),
            (AGENT4, 0, 0),
            (AGENT5, 0, 0),
        ],
        &[
            (AGENT0, 2, 2),
            (AGENT1, 1, 1),
            (AGENT2, 1, 1),
            (AGENT3, 1, 1),
            (AGENT4, 1, 1),
            (AGENT5, 1, 1),
        ],
    );
    local_test(
        (Some(3), Some(3)),
        &[
            (AGENT0, 0, 0),
            (AGENT1, 0, 0),
            (AGENT2, 0, 0),
            (AGENT3, 0, 0),
            (AGENT4, 0, 0),
            (AGENT5, 0, 0),
        ],
        &[
            (AGENT0, 1, 1),
            (AGENT1, 1, 1),
            (AGENT2, 1, 1),
            (AGENT3, 0, 0),
            (AGENT4, 0, 0),
            (AGENT5, 0, 0),
        ],
    );
    local_test(
        (Some(3), Some(3)),
        &[
            (AGENT0, 0, 0),
            (AGENT1, 0, 0),
            (AGENT2, 0, 0),
            (AGENT3, 0, 0),
            (AGENT4, 0, 0),
            (AGENT5, 0, 0),
        ],
        &[
            (AGENT0, 1, 1),
            (AGENT1, 1, 1),
            (AGENT2, 1, 1),
            (AGENT3, 0, 0),
            (AGENT4, 0, 0),
            (AGENT5, 0, 0),
        ],
    );
    local_test(
        (Some(3), Some(3)),
        &[
            (AGENT0, 0, 0),
            (AGENT1, 1, 1),
            (AGENT2, 1, 1),
            (AGENT3, 1, 1),
            (AGENT4, 0, 0),
            (AGENT5, 0, 0),
        ],
        &[
            (AGENT0, 1, 1),
            (AGENT1, 0, 0),
            (AGENT2, 0, 0),
            (AGENT3, 0, 0),
            (AGENT4, 1, 1),
            (AGENT5, 1, 1),
        ],
    );
}
#[test]
fn test_check_valid_agents_get_extra_tasks_eq_mode() {
    let store = CwCroncat::default();
    let mut deps = mock_dependencies_with_balance(&coins(200, NATIVE_DENOM));
    let env = mock_env();
    let mut balancer = RoundRobinBalancer::new(BalancerMode::Equalizer);
    let config = mock_config();

    store.config.save(&mut deps.storage, &config).unwrap();

    let mut active_agents: Vec<Addr> = store
        .agent_active_queue
        .may_load(&deps.storage)
        .unwrap()
        .unwrap_or_default();
    active_agents.extend(vec![
        Addr::unchecked(AGENT0),
        Addr::unchecked(AGENT1),
        Addr::unchecked(AGENT2),
        Addr::unchecked(AGENT3),
        Addr::unchecked(AGENT4),
    ]);

    store
        .agent_active_queue
        .save(&mut deps.storage, &active_agents)
        .unwrap();

    let task_info = TaskInfo {
        task: default_task(),
        task_hash: "".as_bytes().to_vec(),
        task_is_extra: Some(true),
        agent_id: Addr::unchecked(AGENT0),
        slot_kind: SlotType::Block,
    };

    //Notify agent got 1 task
    balancer
        .on_task_completed(
            &mut deps.storage,
            &env,
            &store.config,
            &store.agent_active_queue,
            &task_info,
        )
        .unwrap();

    //Verify agent0 gets extra
    let slot: (Option<u64>, Option<u64>) = (Some(7), Some(7));
    let result = balancer
        .get_agent_tasks(
            &deps.as_ref(),
            &env.clone(),
            &store.config,
            &store.agent_active_queue,
            Addr::unchecked(AGENT0),
            slot,
        )
        .unwrap()
        .unwrap();

    //In equalizer mode, agent0 get 2 task and 1 extra
    assert_eq!(result.num_block_tasks.u64(), 2);
    assert_eq!(result.num_cron_tasks.u64(), 2);
    assert_eq!(result.num_block_tasks_extra.u64(), 1);
    assert_eq!(result.num_cron_tasks_extra.u64(), 1);

    //Verify agent1 gets extra
    let result = balancer
        .get_agent_tasks(
            &deps.as_ref(),
            &env.clone(),
            &store.config,
            &store.agent_active_queue,
            Addr::unchecked(AGENT1),
            slot,
        )
        .unwrap()
        .unwrap();

    assert_eq!(result.num_block_tasks.u64(), 2);
    assert_eq!(result.num_cron_tasks.u64(), 2);
    assert_eq!(result.num_block_tasks_extra.u64(), 1);
    assert_eq!(result.num_cron_tasks_extra.u64(), 1);

    //Verify agent2 gets 1
    let result = balancer
        .get_agent_tasks(
            &deps.as_ref(),
            &env.clone(),
            &store.config,
            &store.agent_active_queue,
            Addr::unchecked(AGENT2),
            slot,
        )
        .unwrap()
        .unwrap();

    assert_eq!(result.num_block_tasks.u64(), 1);
    assert_eq!(result.num_cron_tasks.u64(), 1);
    assert_eq!(result.num_block_tasks_extra.u64(), 0);
    assert_eq!(result.num_cron_tasks_extra.u64(), 0);

    //Verify agent3 not getting extra
    let result = balancer
        .get_agent_tasks(
            &deps.as_ref(),
            &env.clone(),
            &store.config,
            &store.agent_active_queue,
            Addr::unchecked(AGENT3),
            slot,
        )
        .unwrap()
        .unwrap();

    assert_eq!(result.num_block_tasks.u64(), 1);
    assert_eq!(result.num_cron_tasks.u64(), 1);
    assert_eq!(result.num_block_tasks_extra.u64(), 0);
    assert_eq!(result.num_cron_tasks_extra.u64(), 0);
}
#[test]
fn test_on_task_completed() {
    let store = CwCroncat::default();
    let mut deps = mock_dependencies_with_balance(&coins(200, NATIVE_DENOM));
    let env = mock_env();
    let balancer = RoundRobinBalancer::default();
    let mut config = mock_config();

    store.config.save(&mut deps.storage, &config).unwrap();

    let mut active_agents: Vec<Addr> = store
        .agent_active_queue
        .may_load(&deps.storage)
        .unwrap()
        .unwrap_or_default();
    active_agents.extend(vec![
        Addr::unchecked(AGENT0),
        Addr::unchecked(AGENT1),
        Addr::unchecked(AGENT2),
        Addr::unchecked(AGENT3),
        Addr::unchecked(AGENT4),
    ]);

    store
        .agent_active_queue
        .save(&mut deps.storage, &active_agents)
        .unwrap();

    let task_info = TaskInfo {
        task: default_task(),
        task_hash: "".as_bytes().to_vec(),
        task_is_extra: Some(true),
        agent_id: Addr::unchecked(AGENT0),
        slot_kind: SlotType::Block,
    };

    balancer.update_or_append(&mut config.agent_active_indices, (SlotType::Block, 0, 10));
    store.config.save(&mut deps.storage, &config).unwrap();
    balancer
        .on_task_completed(
            &mut deps.storage,
            &env,
            &store.config,
            &store.agent_active_queue,
            &task_info,
        )
        .unwrap();

    config = store.config.load(&mut deps.storage).unwrap();
    assert_eq!(config.agent_active_indices, vec![(SlotType::Block, 0, 11)])
}

#[test]
fn test_on_agent_unregister() {
    let store = CwCroncat::default();
    let mut deps = mock_dependencies_with_balance(&coins(200, NATIVE_DENOM));
    let balancer = RoundRobinBalancer::default();
    let mut config = mock_config();

    store.config.save(&mut deps.storage, &config).unwrap();

    let mut active_agents: Vec<Addr> = store
        .agent_active_queue
        .may_load(&deps.storage)
        .unwrap()
        .unwrap_or_default();
    active_agents.extend(vec![
        Addr::unchecked(AGENT0),
        Addr::unchecked(AGENT1),
        Addr::unchecked(AGENT2),
        Addr::unchecked(AGENT3),
        Addr::unchecked(AGENT4),
    ]);

    store
        .agent_active_queue
        .save(&mut deps.storage, &active_agents)
        .unwrap();

    balancer.update_or_append(&mut config.agent_active_indices, (SlotType::Block, 0, 1));
    balancer.update_or_append(&mut config.agent_active_indices, (SlotType::Cron, 0, 1));
    store.config.save(&mut deps.storage, &config).unwrap();
    balancer
        .on_agent_unregister(
            &mut deps.storage,
            &store.config,
            &store.agent_active_queue,
            Addr::unchecked(AGENT0),
        )
        .unwrap();

    config = store.config.load(&mut deps.storage).unwrap();
    assert_eq!(config.agent_active_indices, vec![])
}
