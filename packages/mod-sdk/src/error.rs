use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ModError {
    #[error("Contract doesn't support execute messages")]
    Noop,
}
