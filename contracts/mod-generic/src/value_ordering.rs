use cosmwasm_schema::cw_serde;

use cosmwasm_std::{StdError, StdResult, Uint512};
use serde_cw_value::Value;

#[cw_serde]
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
