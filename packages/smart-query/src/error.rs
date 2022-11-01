use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum SmartQueryError {
    #[error("Missing placeholder")]
    MissingPlaceholder {},
}
