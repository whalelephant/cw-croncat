use cosmwasm_std::Timestamp;
use croncat_sdk_tasks::types::{Boundary, BoundaryValidated, Interval};

use crate::ContractError;

pub fn validate_boundary(boundary: Option<Boundary>) -> Result<BoundaryValidated, ContractError> {
    let pre_validated = match boundary {
        Some(Boundary::Height { start, end }) => BoundaryValidated {
            start: start.map(Into::into),
            end: end.map(Into::into),
            is_block_boundary: true,
        },
        Some(Boundary::Time { start, end }) => BoundaryValidated {
            start: start.map(|s| s.nanos()),
            end: end.map(|e| e.nanos()),
            is_block_boundary: false,
        },
        None => BoundaryValidated {
            start: None,
            end: None,
            is_block_boundary: true,
        },
    };
    
    if let (Some(start), Some(end)) = (pre_validated.start, pre_validated.end) {
        Err(ContractError::InvalidBoundary {})
    } else {
        Ok(pre_validated)
    }
}
