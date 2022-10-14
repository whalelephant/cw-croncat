use crate::CwCroncat;

use cosmwasm_std::{
    coins,
    testing::{mock_dependencies_with_balance, mock_env},
};
use cw_croncat_core::{
    traits::Intervals,
    types::{BoundaryValidated, Interval, SlotType},
};

use super::helpers::TWO_MINUTES;

#[test]
fn interval_get_next_block_limited() {
    // (input, input, outcome, outcome)
    // test the case when slot_granularity_block == 1
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
        let (next_id, slot_kind) = interval.next(&env, boundary.clone(), 1, 1);
        assert_eq!(outcome_block, &next_id);
        assert_eq!(outcome_slot_kind, &slot_kind);
    }

    // check with slot_granularity_block = 5
    let cases: Vec<(Interval, BoundaryValidated, u64, SlotType)> = vec![
        // Once cases
        (
            Interval::Once,
            BoundaryValidated {
                start: None,
                end: None,
            },
            12350,
            SlotType::Block,
        ),
        (
            Interval::Once,
            BoundaryValidated {
                start: Some(12351),
                end: None,
            },
            12350,
            SlotType::Block,
        ),
        (
            Interval::Once,
            BoundaryValidated {
                start: Some(12348),
                end: None,
            },
            12350,
            SlotType::Block,
        ),
        (
            Interval::Once,
            BoundaryValidated {
                start: None,
                end: Some(12352),
            },
            12350,
            SlotType::Block,
        ),
        (
            Interval::Once,
            BoundaryValidated {
                start: None,
                end: Some(12355),
            },
            12350,
            SlotType::Block,
        ),
        (
            Interval::Once,
            BoundaryValidated {
                start: None,
                end: Some(12346),
            },
            12345,
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
            12350,
            SlotType::Block,
        ),
        (
            Interval::Immediate,
            BoundaryValidated {
                start: Some(12351),
                end: None,
            },
            12350,
            SlotType::Block,
        ),
        (
            Interval::Once,
            BoundaryValidated {
                start: Some(12348),
                end: None,
            },
            12350,
            SlotType::Block,
        ),
        (
            Interval::Once,
            BoundaryValidated {
                start: None,
                end: Some(12352),
            },
            12350,
            SlotType::Block,
        ),
        (
            Interval::Once,
            BoundaryValidated {
                start: None,
                end: Some(12355),
            },
            12350,
            SlotType::Block,
        ),
        (
            Interval::Immediate,
            BoundaryValidated {
                start: None,
                end: Some(12346),
            },
            12345,
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
        let (next_id, slot_kind) = interval.next(&env, boundary.clone(), 5, 1);
        assert_eq!(outcome_block, &next_id);
        assert_eq!(outcome_slot_kind, &slot_kind);
    }
}

#[test]
fn interval_get_next_block_by_offset() {
    // (input, input, outcome, outcome)
    // test the case when slot_granularity_block == 1
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
        // with start
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
        let (next_id, slot_kind) = interval.next(&env, boundary.clone(), 1, 1);
        assert_eq!(outcome_block, &next_id);
        assert_eq!(outcome_slot_kind, &slot_kind);
    }

    // check with slot_granularity_block = 7
    // the slot number is the number of the first block in the block
    let cases: Vec<(Interval, BoundaryValidated, u64, SlotType)> = vec![
        // strictly modulo cases
        (
            Interval::Block(1),
            BoundaryValidated {
                start: None,
                end: None,
            },
            12341,
            SlotType::Block,
        ),
        (
            Interval::Block(10),
            BoundaryValidated {
                start: None,
                end: None,
            },
            12348,
            SlotType::Block,
        ),
        (
            Interval::Block(100),
            BoundaryValidated {
                start: None,
                end: None,
            },
            12397,
            SlotType::Block,
        ),
        (
            Interval::Block(1000),
            BoundaryValidated {
                start: None,
                end: None,
            },
            12999,
            SlotType::Block,
        ),
        (
            Interval::Block(10000),
            BoundaryValidated {
                start: None,
                end: None,
            },
            19999,
            SlotType::Block,
        ),
        (
            Interval::Block(100000),
            BoundaryValidated {
                start: None,
                end: None,
            },
            99995,
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
            12355,
            SlotType::Block,
        ),
        (
            Interval::Block(10),
            BoundaryValidated {
                start: Some(12364),
                end: None,
            },
            12369,
            SlotType::Block,
        ),
        (
            Interval::Block(100),
            BoundaryValidated {
                start: Some(12364),
                end: None,
            },
            12397,
            SlotType::Block,
        ),
        // modulo + boundary end
        (
            Interval::Block(1),
            BoundaryValidated {
                start: None,
                end: Some(12345),
            },
            12341,
            SlotType::Block,
        ),
        (
            Interval::Block(10),
            BoundaryValidated {
                start: None,
                end: Some(12355),
            },
            12348,
            SlotType::Block,
        ),
        (
            Interval::Block(100),
            BoundaryValidated {
                start: None,
                end: Some(12355),
            },
            12299,
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
        let (next_id, slot_kind) = interval.next(&env, boundary.clone(), 7, 1);
        assert_eq!(outcome_block, &next_id);
        assert_eq!(outcome_slot_kind, &slot_kind);
    }
}

#[test]
fn interval_get_next_cron_time() {
    // (input, input, outcome, outcome)
    // test the case when slot_granularity_time == 1
    let cases: Vec<(Interval, BoundaryValidated, u64, SlotType)> = vec![
        (
            Interval::Cron("* * * * * *".to_string()),
            BoundaryValidated {
                start: None,
                end: None,
            },
            1_571_797_420_000_000_000, // current time in nanos is 1_571_797_419_879_305_533
            SlotType::Cron,
        ),
        (
            Interval::Cron("1 * * * * *".to_string()),
            BoundaryValidated {
                start: None,
                end: None,
            },
            1_571_797_441_000_000_000,
            SlotType::Cron,
        ),
        (
            Interval::Cron("* 0 * * * *".to_string()),
            BoundaryValidated {
                start: None,
                end: None,
            },
            1_571_799_600_000_000_000,
            SlotType::Cron,
        ),
        (
            Interval::Cron("15 0 * * * *".to_string()),
            BoundaryValidated {
                start: None,
                end: None,
            },
            1_571_799_615_000_000_000,
            SlotType::Cron,
        ),
        // with start
        (
            Interval::Cron("15 0 * * * *".to_string()),
            BoundaryValidated {
                start: Some(1_471_799_600_000_000_000),
                end: None,
            },
            1_571_799_615_000_000_000,
            SlotType::Cron,
        ),
        (
            Interval::Cron("15 0 * * * *".to_string()),
            BoundaryValidated {
                start: Some(1_571_799_600_000_000_000),
                end: None,
            },
            1_571_799_615_000_000_000,
            SlotType::Cron,
        ),
        (
            Interval::Cron("15 0 * * * *".to_string()),
            BoundaryValidated {
                start: Some(1_571_799_700_000_000_000),
                end: None,
            },
            1_571_803_215_000_000_000,
            SlotType::Cron,
        ),
        // cases when a boundary has end
        // current slot is the end slot
        (
            Interval::Cron("* * * * * *".to_string()),
            BoundaryValidated {
                start: None,
                end: Some(1_571_797_419_879_305_533),
            },
            1_571_797_419_879_305_533,
            SlotType::Cron,
        ),
        // the next slot is after the end, return end slot
        (
            Interval::Cron("* * * * * *".to_string()),
            BoundaryValidated {
                start: None,
                end: Some(1_571_797_419_879_305_535),
            },
            1_571_797_419_879_305_535,
            SlotType::Cron,
        ),
        // next slot in boundaries
        (
            Interval::Cron("* * * * * *".to_string()),
            BoundaryValidated {
                start: None,
                end: Some(1_571_797_420_000_000_000),
            },
            1_571_797_420_000_000_000,
            SlotType::Cron,
        ),
        // the task has ended
        (
            Interval::Cron("* * * * * *".to_string()),
            BoundaryValidated {
                start: None,
                end: Some(1_571_797_419_879_305_532),
            },
            0,
            SlotType::Cron,
        ),
    ];
    for (interval, boundary, outcome_time, outcome_slot_kind) in cases.iter() {
        let env = mock_env();
        // CHECK IT!
        let (next_id, slot_kind) = interval.next(&env, boundary.clone(), 1, 1);
        assert_eq!(outcome_time, &next_id);
        assert_eq!(outcome_slot_kind, &slot_kind);
    }

    // slot_granularity_time == 120_000_000_000 ~ 2 minutes
    let cases: Vec<(Interval, BoundaryValidated, u64, SlotType)> = vec![
        (
            Interval::Cron("* * * * * *".to_string()),
            BoundaryValidated {
                start: None,
                end: None,
            },
            // the timestamp is in the current slot, so we take the next slot
            1_571_797_420_000_000_000_u64.saturating_sub(1_571_797_420_000_000_000 % TWO_MINUTES)
                + TWO_MINUTES, // current time in nanos is 1_571_797_419_879_305_533
            SlotType::Cron,
        ),
        (
            Interval::Cron("1 * * * * *".to_string()),
            BoundaryValidated {
                start: None,
                end: None,
            },
            1_571_797_440_000_000_000,
            SlotType::Cron,
        ),
        (
            Interval::Cron("* 0 * * * *".to_string()),
            BoundaryValidated {
                start: None,
                end: None,
            },
            1_571_799_600_000_000_000,
            SlotType::Cron,
        ),
        (
            Interval::Cron("15 0 * * * *".to_string()),
            BoundaryValidated {
                start: None,
                end: None,
            },
            1_571_799_600_000_000_000,
            SlotType::Cron,
        ),
        // with start
        (
            Interval::Cron("15 0 * * * *".to_string()),
            BoundaryValidated {
                start: Some(1_471_799_600_000_000_000),
                end: None,
            },
            1_571_799_600_000_000_000,
            SlotType::Cron,
        ),
        (
            Interval::Cron("15 0 * * * *".to_string()),
            BoundaryValidated {
                start: Some(1_571_799_600_000_000_000),
                end: None,
            },
            1_571_799_600_000_000_000,
            SlotType::Cron,
        ),
        (
            Interval::Cron("15 0 * * * *".to_string()),
            BoundaryValidated {
                start: Some(1_571_799_700_000_000_000),
                end: None,
            },
            1_571_803_200_000_000_000,
            SlotType::Cron,
        ),
        // cases when a boundary has end
        // boundary end in the current slot
        (
            Interval::Cron("* * * * * *".to_string()),
            BoundaryValidated {
                start: None,
                end: Some(1_571_797_419_879_305_535),
            },
            1_571_797_320_000_000_000,
            SlotType::Cron,
        ),
        // next slot in boundaries
        (
            Interval::Cron("1 * * * * *".to_string()),
            BoundaryValidated {
                start: None,
                end: Some(1_571_797_560_000_000_000),
            },
            1_571_797_440_000_000_000,
            SlotType::Cron,
        ),
        // next slot after the end, return end slot
        (
            Interval::Cron("1 * * * * *".to_string()),
            BoundaryValidated {
                start: None,
                end: Some(1_571_797_420_000_000_000),
            },
            1_571_797_320_000_000_000,
            SlotType::Cron,
        ),
        // the task has ended
        (
            Interval::Cron("* * * * * *".to_string()),
            BoundaryValidated {
                start: None,
                end: Some(1_571_797_419_879_305_532),
            },
            0,
            SlotType::Cron,
        ),
    ];
    for (interval, boundary, outcome_time, outcome_slot_kind) in cases.iter() {
        let env = mock_env();
        // CHECK IT!
        let (next_id, slot_kind) = interval.next(&env, boundary.clone(), 1, TWO_MINUTES);
        assert_eq!(outcome_time, &next_id);
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
        Ok(None),
        store.pop_slot_item(&mut deps.storage, 0, SlotType::Cron)
    );
    assert_eq!(
        Ok(None),
        store.pop_slot_item(&mut deps.storage, 0, SlotType::Block)
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
                .pop_slot_item(&mut deps.storage, 1, SlotType::Cron)
                .unwrap()
                .unwrap()
        );
        assert_eq!(
            *task,
            store
                .pop_slot_item(&mut deps.storage, 1, SlotType::Block)
                .unwrap()
                .unwrap()
        );
    }

    // Slot removed if no hash left
    assert_eq!(
        Ok(None),
        store.pop_slot_item(&mut deps.storage, 1, SlotType::Cron)
    );
    assert_eq!(
        Ok(None),
        store.pop_slot_item(&mut deps.storage, 1, SlotType::Block)
    );
}
