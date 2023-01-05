use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CoreError {
    #[error("Invalid gas input")]
    InvalidGas {},
}
