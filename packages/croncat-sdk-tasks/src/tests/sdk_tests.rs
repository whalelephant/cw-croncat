use cosmwasm_std::Uint64;

use crate::types::{get_next_block_by_offset, BoundaryHeight};

#[test]
fn test_get_next_block_by_offset() {
    let boundary = BoundaryHeight {
        start: Some(Uint64::new(1666000)),
        end: Some(Uint64::new(1666010)),
    };
    let interval = 2;
    let mut list = Vec::new();
    let mut block_height = 1665998;
    for _ in 1..20 {
        let result = get_next_block_by_offset(block_height, &boundary, interval);
        if result.0 > 0 {
            list.push(result.0);
        }
        block_height += 1
    }
    assert_eq!(
        list,
        vec![
            1666000, 1666000, 1666002, 1666002, 1666004, 1666004, 1666006, 1666006, 1666008,
            1666008, 1666010, 1666010, 1666010
        ]
    );

    let block_height = 1665998;

    //pass empty boundary check if getting block_height+interval value
    let empty_boundary = BoundaryHeight {
        start: Some(Uint64::new(block_height)),
        end: None,
    };
    let result = get_next_block_by_offset(block_height, &empty_boundary, interval);
    assert_eq!(block_height + interval, result.0);

    let boundary_with_start = BoundaryHeight {
        start: Some(Uint64::new(1666000)),
        end: None,
    };
    let result = get_next_block_by_offset(block_height, &empty_boundary, interval);
    assert_eq!(boundary_with_start.start.unwrap().u64(), result.0);

    let block_height = 1666008;

    let boundary_with_end = BoundaryHeight {
        start: Some(Uint64::new(block_height)),
        end: Some(Uint64::new(1666010)),
    };
    let result = get_next_block_by_offset(block_height, &boundary_with_end, interval);
    assert_eq!(boundary_with_end.end.unwrap().u64(), result.0);
}
