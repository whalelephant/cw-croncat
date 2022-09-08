use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{StdError, StdResult, Uint512};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValueOrdering {
    UnitAbove,
    UnitAboveEqual,
    UnitBelow,
    UnitBelowEqual,
    Equal,
}

pub trait ValueOrd {
    fn lt(&self, other: &Self) -> StdResult<bool>;
    fn le(&self, other: &Self) -> StdResult<bool>;
    fn bt(&self, other: &Self) -> StdResult<bool>;
    fn be(&self, other: &Self) -> StdResult<bool>;
    fn equal(&self, other: &Self) -> bool;
}

/// Only supporting numbers and big numbers for now
impl ValueOrd for Value {
    fn lt(&self, other: &Self) -> StdResult<bool> {
        if let Value::String(strnum) = self {
            let bigint: Uint512 = strnum.parse()?;
            let oth_bigint = other.as_str();
            if let Some(oth) = oth_bigint {
                return Ok(bigint < oth.parse()?);
            }
        } else {
            let num = self.as_u64();
            let oth_num = other.as_u64();
            if let (Some(n), Some(oth)) = (num, oth_num) {
                return Ok(n < oth);
            }
        };

        Err(StdError::parse_err(
            "number",
            "Failed to parse to Uint512 and to u64",
        ))
    }

    fn le(&self, other: &Self) -> StdResult<bool> {
        if let Value::String(strnum) = self {
            let bigint: Uint512 = strnum.parse()?;
            let oth_bigint = other.as_str();
            if let Some(oth) = oth_bigint {
                return Ok(bigint <= oth.parse()?);
            }
        } else {
            let num = self.as_u64();
            let oth_num = other.as_u64();
            if let (Some(n), Some(oth)) = (num, oth_num) {
                return Ok(n <= oth);
            }
        };

        Err(StdError::parse_err(
            "number",
            "Failed to parse to Uint512 and to u64",
        ))
    }

    fn bt(&self, other: &Self) -> StdResult<bool> {
        if let Value::String(strnum) = self {
            let bigint: Uint512 = strnum.parse()?;
            let oth_bigint = other.as_str();
            if let Some(oth) = oth_bigint {
                return Ok(bigint > oth.parse()?);
            }
        } else {
            let num = self.as_u64();
            let oth_num = other.as_u64();
            if let (Some(n), Some(oth)) = (num, oth_num) {
                return Ok(n > oth);
            }
        };

        Err(StdError::parse_err(
            "number",
            "Failed to parse to Uint512 and to u64",
        ))
    }

    fn be(&self, other: &Self) -> StdResult<bool> {
        if let Value::String(strnum) = self {
            let bigint: Uint512 = strnum.parse()?;
            let oth_bigint = other.as_str();
            if let Some(oth) = oth_bigint {
                return Ok(bigint >= oth.parse()?);
            }
        } else {
            let num = self.as_u64();
            let oth_num = other.as_u64();
            if let (Some(n), Some(oth)) = (num, oth_num) {
                return Ok(n >= oth);
            }
        };

        Err(StdError::parse_err(
            "number",
            "Failed to parse to Uint512 and to u64",
        ))
    }

    fn equal(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::StdError;
    use serde_json::Value;

    use super::ValueOrd;

    #[test]
    fn test_lt() {
        // less
        assert!(Value::from(5_u64).lt(&Value::from(6_u64)).unwrap());
        assert!(Value::from("5").lt(&Value::from("6")).unwrap());
        // equal
        assert!(!Value::from(5_u64).lt(&Value::from(5_u64)).unwrap());
        assert!(!Value::from("5").lt(&Value::from("5")).unwrap());
        // bigger than
        assert!(!Value::from(42_u64).lt(&Value::from(8_u64)).unwrap());
        assert!(!Value::from("42").lt(&Value::from("8")).unwrap());
    }

    #[test]
    fn test_lt_negative() {
        let different_types = Value::from(5_u64).lt(&Value::from("6")).unwrap_err();
        assert!(matches!(different_types, StdError::ParseErr { .. }));

        let different_types = Value::from("5").lt(&Value::from(6_u64)).unwrap_err();
        assert!(matches!(different_types, StdError::ParseErr { .. }));

        let invalid_value = Value::from("foo").lt(&Value::from(6_u64)).unwrap_err();
        assert!(matches!(invalid_value, StdError::GenericErr { .. }));

        let invalid_value = Value::from("5").lt(&Value::from("bar")).unwrap_err();
        assert!(matches!(invalid_value, StdError::GenericErr { .. }));
    }

    #[test]
    fn test_le() {
        // less
        assert!(Value::from(5_u64).le(&Value::from(6_u64)).unwrap());
        assert!(Value::from("5").le(&Value::from("6")).unwrap());
        // equal
        assert!(Value::from(5_u64).le(&Value::from(5_u64)).unwrap());
        assert!(Value::from("5").le(&Value::from("5")).unwrap());
        // bigger than
        assert!(!Value::from(42_u64).le(&Value::from(8_u64)).unwrap());
        assert!(!Value::from("42").le(&Value::from("8")).unwrap());
    }

    #[test]
    fn test_le_negative() {
        let different_types = Value::from(5_u64).le(&Value::from("6")).unwrap_err();
        assert!(matches!(different_types, StdError::ParseErr { .. }));

        let different_types = Value::from("5").le(&Value::from(6_u64)).unwrap_err();
        assert!(matches!(different_types, StdError::ParseErr { .. }));

        let invalid_value = Value::from("foo").le(&Value::from(6_u64)).unwrap_err();
        assert!(matches!(invalid_value, StdError::GenericErr { .. }));

        let invalid_value = Value::from("5").le(&Value::from("bar")).unwrap_err();
        assert!(matches!(invalid_value, StdError::GenericErr { .. }));
    }

    #[test]
    fn test_bt() {
        // less
        assert!(!Value::from(5_u64).bt(&Value::from(6_u64)).unwrap());
        assert!(!Value::from("5").bt(&Value::from("6")).unwrap());
        // equal
        assert!(!Value::from(5_u64).bt(&Value::from(5_u64)).unwrap());
        assert!(!Value::from("5").bt(&Value::from("5")).unwrap());
        // bigger than
        assert!(Value::from(42_u64).bt(&Value::from(8_u64)).unwrap());
        assert!(Value::from("42").bt(&Value::from("8")).unwrap());
    }

    #[test]
    fn test_bt_negative() {
        let different_types = Value::from(5_u64).bt(&Value::from("6")).unwrap_err();
        assert!(matches!(different_types, StdError::ParseErr { .. }));

        let different_types = Value::from("5").bt(&Value::from(6_u64)).unwrap_err();
        assert!(matches!(different_types, StdError::ParseErr { .. }));

        let invalid_value = Value::from("foo").bt(&Value::from(6_u64)).unwrap_err();
        assert!(matches!(invalid_value, StdError::GenericErr { .. }));

        let invalid_value = Value::from("5").bt(&Value::from("bar")).unwrap_err();
        assert!(matches!(invalid_value, StdError::GenericErr { .. }));
    }

    #[test]
    fn test_be() {
        // less
        assert!(!Value::from(5_u64).be(&Value::from(6_u64)).unwrap());
        assert!(!Value::from("5").be(&Value::from("6")).unwrap());
        // equal
        assert!(Value::from(5_u64).be(&Value::from(5_u64)).unwrap());
        assert!(Value::from("5").be(&Value::from("5")).unwrap());
        // bigger than
        assert!(Value::from(42_u64).be(&Value::from(8_u64)).unwrap());
        assert!(Value::from("42").be(&Value::from("8")).unwrap());
    }

    #[test]
    fn test_be_negative() {
        let different_types = Value::from(5_u64).be(&Value::from("6")).unwrap_err();
        assert!(matches!(different_types, StdError::ParseErr { .. }));

        let different_types = Value::from("5").be(&Value::from(6_u64)).unwrap_err();
        assert!(matches!(different_types, StdError::ParseErr { .. }));

        let invalid_value = Value::from("foo").be(&Value::from(6_u64)).unwrap_err();
        assert!(matches!(invalid_value, StdError::GenericErr { .. }));

        let invalid_value = Value::from("5").be(&Value::from("bar")).unwrap_err();
        assert!(matches!(invalid_value, StdError::GenericErr { .. }));
    }

    #[test]
    fn test_equal() {
        // less
        assert!(!Value::from(5_u64).equal(&Value::from(6_u64)));
        assert!(!Value::from("5").equal(&Value::from("6")));
        // equal
        assert!(Value::from(5_u64).equal(&Value::from(5_u64)));
        assert!(Value::from("5").equal(&Value::from("5")));
        // bigger than
        assert!(!Value::from(42_u64).equal(&Value::from(8_u64)));
        assert!(!Value::from("42").equal(&Value::from("8")));

        // Equal can match not only numbers
        assert!(Value::from(r#"{"foo": "bar"}"#).equal(&Value::from(r#"{"foo": "bar"}"#)));
        assert!(!Value::from(r#"{"foo": "bar"}"#).equal(&Value::from(r#"{"bar": "foo"}"#)));
    }
}
