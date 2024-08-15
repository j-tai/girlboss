#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod error;
mod job;
mod manager;
mod return_value;
mod status;
mod tests;

pub use error::{Error, Result};
pub use job::Job;
pub use manager::Girlboss;
pub use return_value::JobReturnValue;
pub use status::{JobStatus, Monitor};
