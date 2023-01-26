#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

mod error;
pub mod msg;
pub mod types;

pub use error::CoreError;
