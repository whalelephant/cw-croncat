use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Invalid gas input")]
    InvalidGas {},
}