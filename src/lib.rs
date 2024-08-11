#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod error;
mod job;
mod manager;
mod status;
mod tests;
mod traits;

pub use error::{Error, Result};
pub use job::Job;
pub use manager::Girlboss;
pub use status::{JobStatus, Monitor};
pub use traits::JobOutput;
