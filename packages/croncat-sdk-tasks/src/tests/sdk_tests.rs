use crate::types::{get_next_block_by_offset, BoundaryValidated};

#[test]
fn test_get_next_block_by_offset() {
    let boundary = BoundaryValidated {
        start: 1666000,
        end: Some(1666010),
        is_block_boundary: true,
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
    let empty_boundary = BoundaryValidated {
        start: block_height,
        end: None,
        is_block_boundary: true,
    };
    let result = get_next_block_by_offset(block_height, &empty_boundary, interval);
    assert_eq!(block_height + interval, result.0);

    let boundary_with_start = BoundaryValidated {
        start: 1666000,
        end: None,
        is_block_boundary: true,
    };
    let result = get_next_block_by_offset(block_height, &empty_boundary, interval);
    assert_eq!(boundary_with_start.start, result.0);

    let block_height = 1666008;

    let boundary_with_end = BoundaryValidated {
        start: block_height,
        end: Some(1666010),
        is_block_boundary: true,
    };
    let result = get_next_block_by_offset(block_height, &boundary_with_end, interval);
    assert_eq!(boundary_with_end.end.unwrap(), result.0);
}
