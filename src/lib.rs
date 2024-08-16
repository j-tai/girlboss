#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod error;
mod job;
mod manager;
mod return_status;
mod status;
mod tests;

pub use error::{Error, Result};
pub use job::Job;
pub use manager::Girlboss;
pub use return_status::JobReturnStatus;
pub use status::{JobStatus, Monitor};
