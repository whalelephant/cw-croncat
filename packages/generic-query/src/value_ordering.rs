use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{StdError, StdResult, Uint512};
use serde_cw_value::Value;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValueOrdering {
    UnitAbove,
    UnitAboveEqual,
    UnitBelow,
    UnitBelowEqual,
    Equal,
    NotEqual,
}

impl ValueOrdering {
    pub fn val_cmp(&self, lhs: &Value, rhs: &Value) -> StdResult<bool> {
        let res = match self {
            ValueOrdering::UnitAbove => lhs.bt_g(rhs)?,
            ValueOrdering::UnitAboveEqual => lhs.be_g(rhs)?,
            ValueOrdering::UnitBelow => lhs.lt_g(rhs)?,
            ValueOrdering::UnitBelowEqual => lhs.le_g(rhs)?,
            ValueOrdering::Equal => lhs.eq(rhs),
            ValueOrdering::NotEqual => lhs.ne(rhs),
        };
        Ok(res)
    }
}

pub trait ValueOrd {
    fn lt_g(&self, other: &Self) -> StdResult<bool>;
    fn le_g(&self, other: &Self) -> StdResult<bool>;
    fn bt_g(&self, other: &Self) -> StdResult<bool>;
    fn be_g(&self, other: &Self) -> StdResult<bool>;
    fn equal(&self, other: &Self) -> bool;
}

/// Only supporting numbers and big numbers for now
impl ValueOrd for Value {
    fn lt_g(&self, other: &Self) -> StdResult<bool> {
        match (self, other) {
            (Value::String(str_num), Value::String(oth)) => {
                let big_num: Uint512 = str_num.parse()?;
                let big_oth: Uint512 = oth.parse()?;
                Ok(big_num < big_oth)
            }
            (Value::U64(n), Value::U64(o)) => Ok(n < o),
            (Value::U32(n), Value::U32(o)) => Ok(n < o),
            (Value::U16(n), Value::U16(o)) => Ok(n < o),
            (Value::U8(n), Value::U8(o)) => Ok(n < o),
            _ => Err(StdError::parse_err(
                "number",
                "Failed to parse to Uint512 and to u64",
            )),
        }
    }

    fn le_g(&self, other: &Self) -> StdResult<bool> {
        match (self, other) {
            (Value::String(str_num), Value::String(oth)) => {
                let big_num: Uint512 = str_num.parse()?;
                let big_oth: Uint512 = oth.parse()?;
                Ok(big_num <= big_oth)
            }
            (Value::U64(n), Value::U64(o)) => Ok(n <= o),
            (Value::U32(n), Value::U32(o)) => Ok(n <= o),
            (Value::U16(n), Value::U16(o)) => Ok(n <= o),
            (Value::U8(n), Value::U8(o)) => Ok(n <= o),
            _ => Err(StdError::parse_err(
                "number",
                "Failed to parse to Uint512 and to u64",
            )),
        }
    }

    fn bt_g(&self, other: &Self) -> StdResult<bool> {
        match (self, other) {
            (Value::String(str_num), Value::String(oth)) => {
                let big_num: Uint512 = str_num.parse()?;
                let big_oth: Uint512 = oth.parse()?;
                Ok(big_num > big_oth)
            }
            (Value::U64(n), Value::U64(o)) => Ok(n > o),
            (Value::U32(n), Value::U32(o)) => Ok(n > o),
            (Value::U16(n), Value::U16(o)) => Ok(n > o),
            (Value::U8(n), Value::U8(o)) => Ok(n > o),
            _ => Err(StdError::parse_err(
                "number",
                "Failed to parse to Uint512 and to u64",
            )),
        }
    }

    fn be_g(&self, other: &Self) -> StdResult<bool> {
        match (self, other) {
            (Value::String(str_num), Value::String(oth)) => {
                let big_num: Uint512 = str_num.parse()?;
                let big_oth: Uint512 = oth.parse()?;
                Ok(big_num >= big_oth)
            }
            (Value::U64(n), Value::U64(o)) => Ok(n >= o),
            (Value::U32(n), Value::U32(o)) => Ok(n >= o),
            (Value::U16(n), Value::U16(o)) => Ok(n >= o),
            (Value::U8(n), Value::U8(o)) => Ok(n >= o),
            _ => Err(StdError::parse_err(
                "number",
                "Failed to parse to Uint512 and to u64",
            )),
        }
    }

    fn equal(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::StdError;

    use super::ValueOrd;

    #[test]
    fn test_lt_g() {
        // less
        assert!(serde_cw_value::to_value(5_u64)
            .unwrap()
            .lt_g(&serde_cw_value::to_value(6_u64).unwrap())
            .unwrap());
        assert!(serde_cw_value::to_value("5")
            .unwrap()
            .lt_g(&serde_cw_value::to_value("6").unwrap())
            .unwrap());
        // equal
        assert!(!serde_cw_value::to_value(5_u64)
            .unwrap()
            .lt_g(&serde_cw_value::to_value(5_u64).unwrap())
            .unwrap());
        assert!(!serde_cw_value::to_value("5")
            .unwrap()
            .lt_g(&serde_cw_value::to_value("5").unwrap())
            .unwrap());
        // bigger than
        assert!(!serde_cw_value::to_value(42_u64)
            .unwrap()
            .lt_g(&serde_cw_value::to_value(8_u64).unwrap())
            .unwrap());
        assert!(!serde_cw_value::to_value("42")
            .unwrap()
            .lt_g(&serde_cw_value::to_value("8").unwrap())
            .unwrap());
    }

    #[test]
    fn test_lt_negative() {
        let different_types = serde_cw_value::to_value(5_u64)
            .unwrap()
            .lt_g(&serde_cw_value::to_value("6").unwrap())
            .unwrap_err();
        assert!(matches!(different_types, StdError::ParseErr { .. }));

        let different_types = serde_cw_value::to_value("5")
            .unwrap()
            .lt_g(&serde_cw_value::to_value(6_u64).unwrap())
            .unwrap_err();
        assert!(matches!(different_types, StdError::ParseErr { .. }));

        let invalid_value = serde_cw_value::to_value("foo")
            .unwrap()
            .lt_g(&serde_cw_value::to_value(6_u64).unwrap())
            .unwrap_err();
        assert!(matches!(invalid_value, StdError::ParseErr { .. }));

        let invalid_value = serde_cw_value::to_value("5")
            .unwrap()
            .lt_g(&serde_cw_value::to_value("bar").unwrap())
            .unwrap_err();
        assert!(matches!(invalid_value, StdError::GenericErr { .. }));
    }

    #[test]
    fn test_le_g() {
        // less
        assert!(serde_cw_value::to_value(5_u64)
            .unwrap()
            .le_g(&serde_cw_value::to_value(6_u64).unwrap())
            .unwrap());
        assert!(serde_cw_value::to_value("5")
            .unwrap()
            .le_g(&serde_cw_value::to_value("6").unwrap())
            .unwrap());
        // equal
        assert!(serde_cw_value::to_value(5_u64)
            .unwrap()
            .le_g(&serde_cw_value::to_value(5_u64).unwrap())
            .unwrap());
        assert!(serde_cw_value::to_value("5")
            .unwrap()
            .le_g(&serde_cw_value::to_value("5").unwrap())
            .unwrap());
        // bigger than
        assert!(!serde_cw_value::to_value(42_u64)
            .unwrap()
            .le_g(&serde_cw_value::to_value(8_u64).unwrap())
            .unwrap());
        assert!(!serde_cw_value::to_value("42")
            .unwrap()
            .le_g(&serde_cw_value::to_value("8").unwrap())
            .unwrap());
    }

    #[test]
    fn test_le_negative() {
        let different_types = serde_cw_value::to_value(5_u64)
            .unwrap()
            .le_g(&serde_cw_value::to_value("6").unwrap())
            .unwrap_err();
        assert!(matches!(different_types, StdError::ParseErr { .. }));

        let different_types = serde_cw_value::to_value("5")
            .unwrap()
            .le_g(&serde_cw_value::to_value(6_u64).unwrap())
            .unwrap_err();
        assert!(matches!(different_types, StdError::ParseErr { .. }));

        let invalid_value = serde_cw_value::to_value("foo")
            .unwrap()
            .le_g(&serde_cw_value::to_value(6_u64).unwrap())
            .unwrap_err();
        assert!(matches!(invalid_value, StdError::ParseErr { .. }));

        let invalid_value = serde_cw_value::to_value("5")
            .unwrap()
            .le_g(&serde_cw_value::to_value("bar").unwrap())
            .unwrap_err();
        assert!(matches!(invalid_value, StdError::GenericErr { .. }));
    }

    #[test]
    fn test_bt_g() {
        // less
        assert!(!serde_cw_value::to_value(5_u64)
            .unwrap()
            .bt_g(&serde_cw_value::to_value(6_u64).unwrap())
            .unwrap());
        assert!(!serde_cw_value::to_value("5")
            .unwrap()
            .bt_g(&serde_cw_value::to_value("6").unwrap())
            .unwrap());
        // equal
        assert!(!serde_cw_value::to_value(5_u64)
            .unwrap()
            .bt_g(&serde_cw_value::to_value(5_u64).unwrap())
            .unwrap());
        assert!(!serde_cw_value::to_value("5")
            .unwrap()
            .bt_g(&serde_cw_value::to_value("5").unwrap())
            .unwrap());
        // bigger than
        assert!(serde_cw_value::to_value(42_u64)
            .unwrap()
            .bt_g(&serde_cw_value::to_value(8_u64).unwrap())
            .unwrap());
        assert!(serde_cw_value::to_value("42")
            .unwrap()
            .bt_g(&serde_cw_value::to_value("8").unwrap())
            .unwrap());
    }

    #[test]
    fn test_bt_negative() {
        let different_types = serde_cw_value::to_value(5_u64)
            .unwrap()
            .bt_g(&serde_cw_value::to_value("6").unwrap())
            .unwrap_err();
        assert!(matches!(different_types, StdError::ParseErr { .. }));

        let different_types = serde_cw_value::to_value("5")
            .unwrap()
            .bt_g(&serde_cw_value::to_value(6_u64).unwrap())
            .unwrap_err();
        assert!(matches!(different_types, StdError::ParseErr { .. }));

        let invalid_value = serde_cw_value::to_value("foo")
            .unwrap()
            .bt_g(&serde_cw_value::to_value(6_u64).unwrap())
            .unwrap_err();
        assert!(matches!(invalid_value, StdError::ParseErr { .. }));

        let invalid_value = serde_cw_value::to_value("5")
            .unwrap()
            .bt_g(&serde_cw_value::to_value("bar").unwrap())
            .unwrap_err();
        assert!(matches!(invalid_value, StdError::GenericErr { .. }));
    }

    #[test]
    fn test_be_g() {
        // less
        assert!(!serde_cw_value::to_value(5_u64)
            .unwrap()
            .be_g(&serde_cw_value::to_value(6_u64).unwrap())
            .unwrap());
        assert!(!serde_cw_value::to_value("5")
            .unwrap()
            .be_g(&serde_cw_value::to_value("6").unwrap())
            .unwrap());
        // equal
        assert!(serde_cw_value::to_value(5_u64)
            .unwrap()
            .be_g(&serde_cw_value::to_value(5_u64).unwrap())
            .unwrap());
        assert!(serde_cw_value::to_value("5")
            .unwrap()
            .be_g(&serde_cw_value::to_value("5").unwrap())
            .unwrap());
        // bigger than
        assert!(serde_cw_value::to_value(42_u64)
            .unwrap()
            .be_g(&serde_cw_value::to_value(8_u64).unwrap())
            .unwrap());
        assert!(serde_cw_value::to_value("42")
            .unwrap()
            .be_g(&serde_cw_value::to_value("8").unwrap())
            .unwrap());
    }

    #[test]
    fn test_be_negative() {
        let different_types = serde_cw_value::to_value(5_u64)
            .unwrap()
            .be_g(&serde_cw_value::to_value("6").unwrap())
            .unwrap_err();
        assert!(matches!(different_types, StdError::ParseErr { .. }));

        let different_types = serde_cw_value::to_value("5")
            .unwrap()
            .be_g(&serde_cw_value::to_value(6_u64).unwrap())
            .unwrap_err();
        assert!(matches!(different_types, StdError::ParseErr { .. }));

        let invalid_value = serde_cw_value::to_value("foo")
            .unwrap()
            .be_g(&serde_cw_value::to_value(6_u64).unwrap())
            .unwrap_err();
        assert!(matches!(invalid_value, StdError::ParseErr { .. }));

        let invalid_value = serde_cw_value::to_value("5")
            .unwrap()
            .be_g(&serde_cw_value::to_value("bar").unwrap())
            .unwrap_err();
        assert!(matches!(invalid_value, StdError::GenericErr { .. }));
    }

    #[test]
    fn test_equal() {
        // less
        assert!(!serde_cw_value::to_value(5_u64)
            .unwrap()
            .equal(&serde_cw_value::to_value(6_u64).unwrap()));
        assert!(!serde_cw_value::to_value("5")
            .unwrap()
            .equal(&serde_cw_value::to_value("6").unwrap()));
        // equal
        assert!(serde_cw_value::to_value(5_u64)
            .unwrap()
            .equal(&serde_cw_value::to_value(5_u64).unwrap()));
        assert!(serde_cw_value::to_value("5")
            .unwrap()
            .equal(&serde_cw_value::to_value("5").unwrap()));
        // bigger than
        assert!(!serde_cw_value::to_value(42_u64)
            .unwrap()
            .equal(&serde_cw_value::to_value(8_u64).unwrap()));
        assert!(!serde_cw_value::to_value("42")
            .unwrap()
            .equal(&serde_cw_value::to_value("8").unwrap()));

        // Equal can match not only numbers
        assert!(serde_cw_value::to_value(r#"{"foo": "bar"}"#)
            .unwrap()
            .equal(&serde_cw_value::to_value(r#"{"foo": "bar"}"#).unwrap()));
        assert!(!serde_cw_value::to_value(r#"{"foo": "bar"}"#)
            .unwrap()
            .equal(&serde_cw_value::to_value(r#"{"bar": "foo"}"#).unwrap()));
    }
}
