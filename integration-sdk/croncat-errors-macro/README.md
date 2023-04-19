# CronCat errors macro

This package creates a procedural attribute macro that creates `#[croncat_error]`, intended to be placed above the integrating contract's error enum, which is conventionally called `ContractError`.

**Note**: this macro should go above the usual `Derive` we see for this enum. The correct placement looks like this:

```rs
#[croncat_error]
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
  // variants here…
}
```

It will throw a compile-time error if it is not placed correctly.

The macro adds this enum variant:

```rs
#[error("CronCat error: {err:?}")]
  CronCatError {
  err: CronCatContractError
}
```

It also adds this logic:

```rs
impl From<CronCatContractError> for ContractError {
  fn from(error: CronCatContractError) -> Self {
    ContractError::CronCatError {
      err: error,
    }
  }
}
```

The above logic will allow helper functions in `croncat-integration-utils` to propagate errors in a manner consistent with the contract's errors.
