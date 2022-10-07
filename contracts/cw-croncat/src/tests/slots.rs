use crate::CwCroncat;

use cosmwasm_std::{
    coins,
    testing::{mock_dependencies_with_balance, mock_env},
};
use cw_croncat_core::{
    traits::Intervals,
    types::{BoundaryValidated, Interval, SlotType},
};

#[test]
fn interval_get_next_block_limited() {
    // (input, input, outcome, outcome)
    let cases: Vec<(Interval, BoundaryValidated, u64, SlotType)> = vec![
        // Once cases
        (
            Interval::Once,
            BoundaryValidated {
                start: None,
                end: None,
            },
            12346,
            SlotType::Block,
        ),
        (
            Interval::Once,
            BoundaryValidated {
                start: Some(12348),
                end: None,
            },
            12348,
            SlotType::Block,
        ),
        (
            Interval::Once,
            BoundaryValidated {
                start: None,
                end: Some(12346),
            },
            12346,
            SlotType::Block,
        ),
        (
            Interval::Once,
            BoundaryValidated {
                start: None,
                end: Some(12340),
            },
            0,
            SlotType::Block,
        ),
        // Immediate cases
        (
            Interval::Immediate,
            BoundaryValidated {
                start: None,
                end: None,
            },
            12346,
            SlotType::Block,
        ),
        (
            Interval::Immediate,
            BoundaryValidated {
                start: Some(12348),
                end: None,
            },
            12348,
            SlotType::Block,
        ),
        (
            Interval::Immediate,
            BoundaryValidated {
                start: None,
                end: Some(12346),
            },
            12346,
            SlotType::Block,
        ),
        (
            Interval::Immediate,
            BoundaryValidated {
                start: None,
                end: Some(12340),
            },
            0,
            SlotType::Block,
        ),
    ];
    for (interval, boundary, outcome_block, outcome_slot_kind) in cases.iter() {
        let env = mock_env();
        // CHECK IT!
        let (next_id, slot_kind) = interval.next(&env, boundary.clone());
        println!("next_id {:?}, slot_kind {:?}", next_id, slot_kind);
        assert_eq!(outcome_block, &next_id);
        assert_eq!(outcome_slot_kind, &slot_kind);
    }
}

#[test]
fn interval_get_next_block_by_offset() {
    // (input, input, outcome, outcome)
    let cases: Vec<(Interval, BoundaryValidated, u64, SlotType)> = vec![
        // strictly modulo cases
        (
            Interval::Block(1),
            BoundaryValidated {
                start: None,
                end: None,
            },
            12346,
            SlotType::Block,
        ),
        (
            Interval::Block(10),
            BoundaryValidated {
                start: None,
                end: None,
            },
            12350,
            SlotType::Block,
        ),
        (
            Interval::Block(100),
            BoundaryValidated {
                start: None,
                end: None,
            },
            12400,
            SlotType::Block,
        ),
        (
            Interval::Block(1000),
            BoundaryValidated {
                start: None,
                end: None,
            },
            13000,
            SlotType::Block,
        ),
        (
            Interval::Block(10000),
            BoundaryValidated {
                start: None,
                end: None,
            },
            20000,
            SlotType::Block,
        ),
        (
            Interval::Block(100000),
            BoundaryValidated {
                start: None,
                end: None,
            },
            100000,
            SlotType::Block,
        ),
        // modulo + boundary start
        (
            Interval::Block(1),
            BoundaryValidated {
                start: Some(12348),
                end: None,
            },
            12348,
            SlotType::Block,
        ),
        (
            Interval::Block(10),
            BoundaryValidated {
                start: Some(12360),
                end: None,
            },
            12360,
            SlotType::Block,
        ),
        (
            Interval::Block(10),
            BoundaryValidated {
                start: Some(12364),
                end: None,
            },
            12370,
            SlotType::Block,
        ),
        (
            Interval::Block(100),
            BoundaryValidated {
                start: Some(12364),
                end: None,
            },
            12400,
            SlotType::Block,
        ),
        // modulo + boundary end
        (
            Interval::Block(1),
            BoundaryValidated {
                start: None,
                end: Some(12345),
            },
            12345,
            SlotType::Block,
        ),
        (
            Interval::Block(10),
            BoundaryValidated {
                start: None,
                end: Some(12355),
            },
            12350,
            SlotType::Block,
        ),
        (
            Interval::Block(100),
            BoundaryValidated {
                start: None,
                end: Some(12355),
            },
            12300,
            SlotType::Block,
        ),
        (
            Interval::Block(100),
            BoundaryValidated {
                start: None,
                end: Some(12300),
            },
            0,
            SlotType::Block,
        ),
    ];
    for (interval, boundary, outcome_block, outcome_slot_kind) in cases.iter() {
        let env = mock_env();
        // CHECK IT!
        let (next_id, slot_kind) = interval.next(&env, boundary.clone());
        assert_eq!(outcome_block, &next_id);
        assert_eq!(outcome_slot_kind, &slot_kind);
    }
}

#[test]
fn slot_items_get_current() {
    let mut deps = mock_dependencies_with_balance(&coins(200, ""));
    let store = CwCroncat::default();
    let mock_env = mock_env();
    let current_block = mock_env.block.height;
    let current_time = mock_env.block.time.nanos();
    let task_hash = "ad15b0f15010d57a51ff889d3400fe8d083a0dab2acfc752c5eb55e9e6281705"
        .as_bytes()
        .to_vec();

    // Check for empty store
    assert_eq!(
        (None, None),
        store.get_current_slot_items(&mock_env.block, &deps.storage, None)
    );

    // Setup of block and cron slots
    store
        .time_slots
        .save(
            &mut deps.storage,
            current_time + 1,
            &vec![task_hash.clone()],
        )
        .unwrap();
    store
        .block_slots
        .save(
            &mut deps.storage,
            current_block + 1,
            &vec![task_hash.clone()],
        )
        .unwrap();

    // Empty if not time/block yet
    assert_eq!(
        (None, None),
        store.get_current_slot_items(&mock_env.block, &deps.storage, None)
    );

    // And returns task when it's time
    let mut mock_env = mock_env;
    mock_env.block.time = mock_env.block.time.plus_nanos(1);
    assert_eq!(
        (None, Some(current_time + 1)),
        store.get_current_slot_items(&mock_env.block, &deps.storage, None)
    );

    // Or later
    mock_env.block.time = mock_env.block.time.plus_nanos(1);
    assert_eq!(
        (None, Some(current_time + 1)),
        store.get_current_slot_items(&mock_env.block, &deps.storage, None)
    );

    // Check, that Block is preferred over cron and block height reached
    mock_env.block.height += 1;
    assert_eq!(
        (Some(current_block + 1), Some(current_time + 1)),
        store.get_current_slot_items(&mock_env.block, &deps.storage, None)
    );

    // Or block(s) ahead
    mock_env.block.height += 1;
    assert_eq!(
        (Some(current_block + 1), Some(current_time + 1)),
        store.get_current_slot_items(&mock_env.block, &deps.storage, None)
    );
}

#[test]
fn slot_items_pop() {
    let mut deps = mock_dependencies_with_balance(&coins(200, ""));
    let mut store = CwCroncat::default();

    // Empty slots
    store
        .time_slots
        .save(&mut deps.storage, 0, &vec![])
        .unwrap();
    store
        .block_slots
        .save(&mut deps.storage, 0, &vec![])
        .unwrap();
    assert_eq!(
        None,
        store.pop_slot_item(&mut deps.storage, &0, &SlotType::Cron)
    );
    assert_eq!(
        None,
        store.pop_slot_item(&mut deps.storage, &0, &SlotType::Block)
    );

    // Just checking mutiple tasks
    let multiple_tasks = vec![
        "task_1".as_bytes().to_vec(),
        "task_2".as_bytes().to_vec(),
        "task_3".as_bytes().to_vec(),
    ];
    store
        .time_slots
        .save(&mut deps.storage, 1, &multiple_tasks)
        .unwrap();
    store
        .block_slots
        .save(&mut deps.storage, 1, &multiple_tasks)
        .unwrap();
    for task in multiple_tasks.iter().rev() {
        assert_eq!(
            *task,
            store
                .pop_slot_item(&mut deps.storage, &1, &SlotType::Cron)
                .unwrap()
        );
        assert_eq!(
            *task,
            store
                .pop_slot_item(&mut deps.storage, &1, &SlotType::Block)
                .unwrap()
        );
    }

    // Slot removed if no hash left
    assert_eq!(
        None,
        store.pop_slot_item(&mut deps.storage, &1, &SlotType::Cron)
    );
    assert_eq!(
        None,
        store.pop_slot_item(&mut deps.storage, &1, &SlotType::Block)
    );
}
