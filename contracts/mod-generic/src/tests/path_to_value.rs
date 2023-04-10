use serde_cw_value::Value;
use serde_cw_value::Value::{Map, Seq};

use crate::types::PathToValue;
use crate::types::ValueIndex::{Index, Key};
use std::collections::BTreeMap;

#[test]
fn test_find_value_empty_path() {
    let mut value = Map(BTreeMap::new());
    let path = PathToValue(vec![]);
    assert_eq!(path.find_value(&mut value.clone()).unwrap(), &mut value);
}

#[test]
fn test_find_value_empty_path_realworld_case() {
    let mut value = Value::String("raw_string_returned".to_string());
    let path = PathToValue(vec![]);
    assert_eq!(path.find_value(&mut value.clone()).unwrap(), &mut value);
}

#[test]
fn test_find_value_single_valid_key() {
    let mut value = Map(BTreeMap::new());
    let key = "key1";
    let mut inner_value = Value::Bool(true);

    if let Map(ref mut map) = value {
        map.insert(Value::String(key.to_string()), inner_value.clone());
    }

    let path = PathToValue(vec![Key(key.to_string())]);
    assert_eq!(path.find_value(&mut value).unwrap(), &mut inner_value);
}

#[test]
fn test_find_value_single_invalid_key() {
    let mut value = Map(BTreeMap::new());
    let key = "invalid_key";
    let path = PathToValue(vec![Key(key.to_string())]);

    assert!(path.find_value(&mut value).is_err());
}

#[test]
fn test_find_value_single_valid_index() {
    let mut value = Seq(vec![Value::Bool(true)]);
    let index = 0;
    let path = PathToValue(vec![Index(index)]);

    assert_eq!(path.find_value(&mut value).unwrap(), &mut Value::Bool(true));
}

#[test]
fn test_find_value_single_invalid_index() {
    let mut value = Seq(vec![Value::Bool(true)]);
    let index = 1;
    let path = PathToValue(vec![Index(index)]);

    assert!(path.find_value(&mut value).is_err());
}

#[test]
fn test_find_value_multiple_valid_keys_and_indices() {
    let mut value = Map(BTreeMap::new());
    let mut inner_map = Map(BTreeMap::new());
    let inner_seq = Seq(vec![Value::Bool(false)]);

    if let Map(ref mut map) = inner_map {
        map.insert(Value::String("inner_key".to_string()), inner_seq);
    }

    if let Map(ref mut map) = value {
        map.insert(Value::String("outer_key".to_string()), inner_map);
    }

    let path = PathToValue(vec![
        Key("outer_key".to_string()),
        Key("inner_key".to_string()),
        Index(0),
    ]);
    assert_eq!(
        path.find_value(&mut value).unwrap(),
        &mut Value::Bool(false)
    );
}

#[test]
fn test_find_value_invalid_key_in_middle() {
    let mut value = Map(BTreeMap::new());
    let inner_map = Map(BTreeMap::new());

    if let Map(ref mut map) = value {
        map.insert(Value::String("outer_key".to_string()), inner_map);
    }

    let path = PathToValue(vec![
        Key("outer_key".to_string()),
        Key("invalid_key".to_string()),
    ]);
    assert!(path.find_value(&mut value).is_err());
}

#[test]
fn test_find_value_valid_key_in_middle() {
    let mut value = Map(BTreeMap::new());
    let mut inner_map = Map(BTreeMap::new());

    if let Map(ref mut map) = inner_map {
        map.insert(Value::String("inner_key".to_string()), Value::Bool(true));
    }

    if let Map(ref mut map) = value {
        map.insert(Value::String("outer_key".to_string()), inner_map);
    }

    let path = PathToValue(vec![
        Key("outer_key".to_string()),
        Key("inner_key".to_string()),
    ]);
    assert_eq!(path.find_value(&mut value).unwrap(), &mut Value::Bool(true));
}

#[test]
fn test_find_value_invalid_index_in_middle() {
    let mut value = Map(BTreeMap::new());
    let inner_seq = Seq(vec![Value::Bool(false)]);

    if let Map(ref mut map) = value {
        map.insert(Value::String("outer_key".to_string()), inner_seq);
    }

    let path = PathToValue(vec![Key("outer_key".to_string()), Index(1)]);
    assert!(path.find_value(&mut value).is_err());
}

#[test]
fn test_find_value_valid_index_in_middle() {
    let mut value = Map(BTreeMap::new());
    let inner_seq = Seq(vec![Value::Bool(false)]);

    if let Map(ref mut map) = value {
        map.insert(Value::String("outer_key".to_string()), inner_seq);
    }

    let path = PathToValue(vec![Key("outer_key".to_string()), Index(0)]);
    assert_eq!(
        path.find_value(&mut value).unwrap(),
        &mut Value::Bool(false)
    );
}

#[test]
fn test_find_value_invalid_key_expect_seq() {
    let mut value = Map(BTreeMap::new());
    let inner_value = Value::Bool(true);

    if let Map(ref mut map) = value {
        map.insert(Value::String("key1".to_string()), inner_value);
    }

    let path = PathToValue(vec![Key("key1".to_string()), Index(0)]);
    assert!(path.find_value(&mut value).is_err());
}

#[test]
fn test_find_value_valid_key_expect_seq() {
    let mut value = Map(BTreeMap::new());
    let inner_seq = Seq(vec![Value::Bool(true)]);

    if let Map(ref mut map) = value {
        map.insert(Value::String("key1".to_string()), inner_seq);
    }

    let path = PathToValue(vec![Key("key1".to_string()), Index(0)]);
    assert_eq!(path.find_value(&mut value).unwrap(), &mut Value::Bool(true));
}

#[test]
fn test_find_value_invalid_index_expect_map() {
    let mut value = Seq(vec![Value::Bool(true)]);
    let index = 0;
    let path = PathToValue(vec![Index(index), Key("key1".to_string())]);

    assert!(path.find_value(&mut value).is_err());
}

#[test]
fn test_find_value_valid_index_expect_map() {
    let mut value = Seq(vec![Map(BTreeMap::new())]);

    if let Seq(ref mut seq) = value {
        if let Map(ref mut map) = seq[0] {
            map.insert(Value::String("key1".to_string()), Value::Bool(true));
        }
    }

    let index = 0;
    let path = PathToValue(vec![Index(index), Key("key1".to_string())]);
    assert_eq!(path.find_value(&mut value).unwrap(), &mut Value::Bool(true));
}

#[test]
fn test_find_value_complex_path() {
    let mut value = Map(BTreeMap::new());

    let mut inner_map = Map(BTreeMap::new());
    let mut inner_seq = Seq(vec![Value::Bool(true), Value::Bool(false)]);
    let mut inner_inner_map = Map(BTreeMap::new());
    let mut target_value = Value::String("found".to_string());

    if let Map(ref mut map) = inner_inner_map {
        map.insert(
            Value::String("inner_inner_key".to_string()),
            target_value.clone(),
        );
    }

    if let Seq(ref mut seq) = inner_seq {
        seq.insert(1, inner_inner_map);
    }

    if let Map(ref mut map) = inner_map {
        map.insert(Value::String("inner_key".to_string()), inner_seq);
    }

    if let Map(ref mut map) = value {
        map.insert(Value::String("outer_key".to_string()), inner_map);
    }

    let path = PathToValue(vec![
        Key("outer_key".to_string()),
        Key("inner_key".to_string()),
        Index(1),
        Key("inner_inner_key".to_string()),
    ]);

    assert_eq!(path.find_value(&mut value).unwrap(), &mut target_value);
}
