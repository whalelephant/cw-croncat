use croncat_sdk_agents::types::Config;

use crate::distro::AgentDistributor;
use crate::tests::common::*;
use cosmwasm_std::testing::{
    mock_dependencies_with_balance, mock_env, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{coins, Addr, Empty, Env, MemoryStorage, OwnedDeps, Storage};

use super::common::{mock_config, NATIVE_DENOM};

#[test]
fn test_check_valid_agents_get_tasks() {
    let mut deps: OwnedDeps<
        MemoryStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
    > = mock_dependencies_with_balance(&coins(200, NATIVE_DENOM));
    let mut env = mock_env();
    let mut config = mock_config(String::new().as_str());
    config.min_tasks_per_agent = 1;
    let agent_distributor = AgentDistributor::new();
    agent_distributor
        .reset_nomination_checkpoint(&mut deps.storage)
        .unwrap();
    #[allow(clippy::type_complexity)]
    let cases: &[((u64, u64), &[(&str, u64, u64)], &[(&str, u64, u64)])] = &[
        (
            (7, 7),
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
            (3, 3),
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
            (3, 3),
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
            (3, 3),
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
            (1, 1),
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
            (3, 0),
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
            (0, 3),
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
            (4, 6),
            &[(AGENT0, 0, 0), (AGENT1, 0, 0), (AGENT2, 0, 0)],
            &[(AGENT0, 2, 2), (AGENT1, 1, 2), (AGENT2, 1, 2)],
        ),
        (
            (0, 0),
            &[(AGENT0, 0, 0), (AGENT1, 0, 0), (AGENT2, 0, 0)],
            &[(AGENT0, 0, 0), (AGENT1, 0, 0), (AGENT2, 0, 0)],
        ),
        (
            (23, 37),
            &[(AGENT0, 0, 0), (AGENT1, 0, 0), (AGENT2, 0, 0)],
            &[(AGENT0, 8, 13), (AGENT1, 8, 12), (AGENT2, 7, 12)],
        ),
        (
            (345, 897),
            &[(AGENT0, 0, 0), (AGENT1, 0, 0), (AGENT2, 0, 0)],
            &[(AGENT0, 115, 299), (AGENT1, 115, 299), (AGENT2, 115, 299)],
        ),
    ];

    for case in cases {
        assert_balancer_tasks(
            &mut deps,
            &mut env,
            &mut config,
            &agent_distributor,
            case.0,
            case.1,
            case.2,
        );
    }
}

#[test]
fn test_on_task_completed() {
    let mut deps = mock_dependencies_with_balance(&coins(200, NATIVE_DENOM));
    let agent_distributor = AgentDistributor::new();
    let env = mock_env();

    create_sample_agents(&mut deps.storage, &env, &agent_distributor);
    let agent0_addr = Addr::unchecked(AGENT0);
    for _ in 0..5 {
        agent_distributor
            .notify_task_completed(&mut deps.storage, &env, agent0_addr.clone(), true)
            .unwrap();
    }

    agent_distributor
        .notify_task_completed(&mut deps.storage, &env, agent0_addr.clone(), false)
        .unwrap();

    let agent0 = agent_distributor
        .get_agent(&mut deps.storage, &agent0_addr)
        .unwrap()
        .unwrap();

    assert_eq!(agent0.completed_block_tasks, 5);
    assert_eq!(agent0.completed_cron_tasks, 1);
    assert_eq!(agent0.last_executed_slot, env.block.height);
}

#[test]
fn test_on_agent_unregister() {
    let mut deps = mock_dependencies_with_balance(&coins(200, NATIVE_DENOM));
    let env = mock_env();
    let agent_distributor = AgentDistributor::new();

    create_sample_agents(&mut deps.storage, &env, &agent_distributor);

    let agent0_addr = &Addr::unchecked(AGENT0);
    let agent1_addr = &Addr::unchecked(AGENT1);

    agent_distributor
        .notify_task_completed(&mut deps.storage, &env, agent0_addr.clone(), true)
        .unwrap();
    agent_distributor
        .notify_task_completed(&mut deps.storage, &env, agent1_addr.clone(), true)
        .unwrap();

    agent_distributor
        .remove_agent(&mut deps.storage, agent1_addr)
        .unwrap();
    let agent0 = agent_distributor
        .get_agent(&mut deps.storage, &agent0_addr)
        .unwrap();
    let agent1 = agent_distributor
        .get_agent(&mut deps.storage, &agent1_addr)
        .unwrap();

    assert!(agent0.is_some());
    assert!(agent1.is_none());
}

//helper functions
///Asserts if balancer get the expected amount of tasks with specified active agents and task slots
///
/// # Arguments
///
/// * `slots` - Task slots
/// * `act_agents` - (Address,block_tasks,cron_tasks)
/// * `expected` - (Address,block_tasks,cron_tasks)
fn assert_balancer_tasks(
    deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
    env: &mut Env,
    config: &mut Config,
    distributor: &AgentDistributor,
    slots: (u64, u64),
    act_agents: &[(&str, u64, u64)],
    expected: &[(&str, u64, u64)],
) {
    //Remove agent registered in prev case
    for agent_id_str in &[AGENT0, AGENT1, AGENT2, AGENT3, AGENT4, AGENT5, AGENT6] {
        distributor
            .remove_agent(&mut deps.storage, &Addr::unchecked(agent_id_str.clone()))
            .unwrap();
    }

    //Create active agents
    for (aid, _, _) in act_agents {
        create_active_agent(&mut deps.storage, env, config, distributor, aid);
        if *aid != AGENT0 {
            try_nominate(&mut deps.storage, env, config, &distributor, aid);
        }
    }
    //Check we have all active agents
    assert_eq!(
        distributor.agents_active(&mut deps.storage).unwrap().len(),
        act_agents.len()
    );

    //Some task advancement
    distributor
        .notify_task_created(&mut deps.storage, env, Some(slots.0 + slots.1))
        .unwrap();

    //Task completion
    let mut result = Vec::<(&str, u64, u64)>::new();
    act_agents.iter().for_each(|f| {
        if f.1 > 0 {
            distributor
                .notify_task_completed(&mut deps.storage, &env, Addr::unchecked(f.0), true)
                .unwrap();
        }
        if f.2 > 0 {
            distributor
                .notify_task_completed(&mut deps.storage, &env, Addr::unchecked(f.0), false)
                .unwrap();
        }
    });

    for (agent_id, _, _) in act_agents {
        let (block, cron) = distributor
            .get_available_tasks(&mut deps.storage, &Addr::unchecked(agent_id.clone()), slots)
            .unwrap();
        result.push((agent_id, block, cron));
    }

    assert_eq!(&result, expected);
}

fn create_sample_agents(
    storage: &mut dyn Storage,
    env: &Env,
    agent_distributor: &AgentDistributor,
) {
    agent_distributor
        .add_new_agent(
            storage,
            &env,
            Addr::unchecked(AGENT0),
            Addr::unchecked(PARTICIPANT0),
        )
        .unwrap();

    agent_distributor
        .add_new_agent(
            storage,
            &env,
            Addr::unchecked(AGENT1),
            Addr::unchecked(PARTICIPANT1),
        )
        .unwrap();
    agent_distributor
        .add_new_agent(
            storage,
            &env,
            Addr::unchecked(AGENT2),
            Addr::unchecked(PARTICIPANT2),
        )
        .unwrap();
    agent_distributor
        .add_new_agent(
            storage,
            &env,
            Addr::unchecked(AGENT3),
            Addr::unchecked(PARTICIPANT3),
        )
        .unwrap();
    agent_distributor
        .add_new_agent(
            storage,
            &env,
            Addr::unchecked(AGENT4),
            Addr::unchecked(PARTICIPANT4),
        )
        .unwrap();
    agent_distributor
        .add_new_agent(
            storage,
            &env,
            Addr::unchecked(AGENT5),
            Addr::unchecked(PARTICIPANT5),
        )
        .unwrap();
}

fn create_active_agent(
    storage: &mut dyn Storage,
    env: &mut Env,
    _config: &Config,
    distributor: &AgentDistributor,
    agent_id_str: &str,
) {
    distributor
        .add_new_agent(
            storage,
            &env,
            Addr::unchecked(agent_id_str),
            Addr::unchecked(PARTICIPANT5),
        )
        .unwrap();
}
fn try_nominate(
    storage: &mut dyn Storage,
    env: &mut Env,
    config: &Config,
    distributor: &AgentDistributor,
    agent_id_str: &str,
) {
    //fake task created event for reinit nomination checkpoint
    distributor
        .notify_task_created(storage, env, Some(20))
        .unwrap(); 
    increment_block_height(&mut env.block, Some(20));

    distributor
        .try_nominate_agent(storage, env, config, Addr::unchecked(agent_id_str))
        .unwrap();
}
