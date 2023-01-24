use crate::msg::*;
use croncat_sdk_tasks::types::SlotType;

use crate::distributor::{AgentTaskDistributor, RoundRobinAgentTaskDistributor};
use crate::state::{AGENTS_ACTIVE, AGENT_STATS};
use crate::tests::common::{AGENT0, AGENT1, AGENT2, AGENT3, AGENT4, AGENT5};
use cosmwasm_std::testing::{
    mock_dependencies_with_balance, mock_env, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{coins, Addr, Empty, Env, MemoryStorage, OwnedDeps};

use super::common::{mock_config, NATIVE_DENOM};

///Asserts if balancer get the expected amount of tasks with specified active agents and task slots
///
/// # Arguments
///
/// * `slots` - Task slots
/// * `act_agents` - (Address,block_tasks,cron_tasks)
/// * `expected` - (Address,block_tasks,cron_tasks)
fn assert_balancer_tasks(
    deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
    env: &Env,
    _config: &mut Config,
    slots: (Option<u64>, Option<u64>),
    act_agents: &[(&str, u64, u64)],
    expected: &[(&str, u64, u64)],
) {
    let task_distributor = AgentTaskDistributor::new();
    let mut result = Vec::<(&str, u64, u64)>::new();

    AGENTS_ACTIVE.remove(&mut deps.storage);
    AGENT_STATS.clear(&mut deps.storage);

    let mut active_agents: Vec<Addr> = AGENTS_ACTIVE
        .may_load(&deps.storage)
        .unwrap()
        .unwrap_or_default();
    active_agents.extend(act_agents.iter().map(|mapped| Addr::unchecked(mapped.0)));

    AGENTS_ACTIVE
        .save(&mut deps.storage, &active_agents)
        .unwrap();
    act_agents.iter().for_each(|f| {
        if f.1 > 0 {
            task_distributor
                .on_task_completed(
                    &mut deps.storage,
                    &env,
                    &Addr::unchecked(f.0),
                    SlotType::Block,
                )
                .unwrap();
        }
        if f.2 > 0 {
            task_distributor
                .on_task_completed(
                    &mut deps.storage,
                    &env,
                    &Addr::unchecked(f.0),
                    SlotType::Cron,
                )
                .unwrap();
        }
    });

    for a in act_agents {
        let balancer_result = task_distributor
            .get_agent_tasks(&deps.as_ref(), &env.clone(), Addr::unchecked(a.0), slots)
            .unwrap()
            .unwrap();
        result.push((
            a.0,
            balancer_result.num_block_tasks.u64(),
            balancer_result.num_cron_tasks.u64(),
        ));
    }

    assert_eq!(expected, &result);
}
//EQ Mode
#[test]
fn test_check_valid_agents_get_tasks_eq_mode() {
    let mut deps: OwnedDeps<
        MemoryStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
    > = mock_dependencies_with_balance(&coins(200, NATIVE_DENOM));
    let env = mock_env();
    let mut config = mock_config();

    let cases: &[(
        (Option<u64>, Option<u64>),
        &[(&str, u64, u64)],
        &[(&str, u64, u64)],
    )] = &[
        (
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
        ),
        (
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
        ),
        (
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
        ),
        (
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
        ),
        (
            (Some(1), Some(1)),
            &[
                (AGENT0, 1, 1),
                (AGENT1, 1, 1),
                (AGENT2, 1, 1),
                (AGENT3, 1, 1),
                (AGENT4, 1, 1),
                (AGENT5, 0, 0),
            ],
            &[
                (AGENT0, 0, 0),
                (AGENT1, 0, 0),
                (AGENT2, 0, 0),
                (AGENT3, 0, 0),
                (AGENT4, 0, 0),
                (AGENT5, 1, 1),
            ],
        ),
        (
            (Some(3), Some(0)),
            &[
                (AGENT0, 1, 1),
                (AGENT1, 1, 1),
                (AGENT2, 1, 1),
                (AGENT3, 0, 0),
                (AGENT4, 0, 0),
                (AGENT5, 0, 0),
            ],
            &[
                (AGENT0, 0, 0),
                (AGENT1, 0, 0),
                (AGENT2, 0, 0),
                (AGENT3, 1, 0),
                (AGENT4, 1, 0),
                (AGENT5, 1, 0),
            ],
        ),
        (
            (Some(0), Some(3)),
            &[
                (AGENT0, 1, 1),
                (AGENT1, 1, 1),
                (AGENT2, 1, 1),
                (AGENT3, 0, 0),
                (AGENT4, 0, 0),
                (AGENT5, 0, 0),
            ],
            &[
                (AGENT0, 0, 0),
                (AGENT1, 0, 0),
                (AGENT2, 0, 0),
                (AGENT3, 0, 1),
                (AGENT4, 0, 1),
                (AGENT5, 0, 1),
            ],
        ),
        (
            (Some(4), Some(6)),
            &[(AGENT0, 0, 0), (AGENT1, 0, 0), (AGENT2, 0, 0)],
            &[(AGENT0, 2, 2), (AGENT1, 1, 2), (AGENT2, 1, 2)],
        ),
        (
            (Some(0), Some(0)),
            &[(AGENT0, 0, 0), (AGENT1, 0, 0), (AGENT2, 0, 0)],
            &[(AGENT0, 0, 0), (AGENT1, 0, 0), (AGENT2, 0, 0)],
        ),
        (
            (Some(23), Some(37)),
            &[(AGENT0, 0, 0), (AGENT1, 0, 0), (AGENT2, 0, 0)],
            &[(AGENT0, 8, 13), (AGENT1, 8, 12), (AGENT2, 7, 12)],
        ),
        (
            (Some(345), Some(897)),
            &[(AGENT0, 0, 0), (AGENT1, 0, 0), (AGENT2, 0, 0)],
            &[(AGENT0, 115, 299), (AGENT1, 115, 299), (AGENT2, 115, 299)],
        ),
    ];

    for case in cases {
        assert_balancer_tasks(&mut deps, &env, &mut config, case.0, case.1, case.2);
    }
}

#[test]
fn test_on_task_completed() {
    let mut deps = mock_dependencies_with_balance(&coins(200, NATIVE_DENOM));
    let env = mock_env();
    let task_distributor = AgentTaskDistributor::default();

    let mut active_agents: Vec<Addr> = AGENTS_ACTIVE
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

    AGENTS_ACTIVE
        .save(&mut deps.storage, &active_agents)
        .unwrap();

    let agent0_addr = &Addr::unchecked(AGENT0);
    for _ in 0..5 {
        task_distributor
            .on_task_completed(&mut deps.storage, &env, &agent0_addr, SlotType::Block)
            .unwrap();
    }

    task_distributor
        .on_task_completed(&mut deps.storage, &env, &agent0_addr, SlotType::Cron)
        .unwrap();

    let stats = AGENT_STATS.load(&mut deps.storage, &agent0_addr).unwrap();
    assert_eq!(stats.completed_block_tasks, 5);
    assert_eq!(stats.completed_cron_tasks, 1);
}

#[test]
fn test_on_agent_unregister() {
    let mut deps = mock_dependencies_with_balance(&coins(200, NATIVE_DENOM));
    let env = mock_env();
    let task_distributor = AgentTaskDistributor::default();

    let mut active_agents: Vec<Addr> = AGENTS_ACTIVE
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

    AGENTS_ACTIVE
        .save(&mut deps.storage, &active_agents)
        .unwrap();

    let agent0_addr = &Addr::unchecked(AGENT0);
    let agent1_addr = &Addr::unchecked(AGENT1);

    task_distributor
        .on_task_completed(&mut deps.storage, &env, &agent0_addr, SlotType::Block)
        .unwrap();
    task_distributor
        .on_task_completed(&mut deps.storage, &env, &agent1_addr, SlotType::Block)
        .unwrap();

    task_distributor
        .on_agent_unregistered(&mut deps.storage, &agent1_addr)
        .unwrap();

    let stats0 = AGENT_STATS.load(&mut deps.storage, &agent0_addr);
    let stats1 = AGENT_STATS.load(&mut deps.storage, &agent1_addr);

    assert!(stats0.is_ok());
    assert!(stats1.is_err());
}
