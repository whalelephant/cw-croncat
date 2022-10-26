use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum SmartQueryError {
    #[error("Missing placeholder")]
    MissingPlaceholder {},
}