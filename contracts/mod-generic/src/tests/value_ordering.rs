use cosmwasm_std::StdError;

use crate::value_ordering::ValueOrd;

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
