//! Runtime-agnostic types.
//!
//! **Note**: you very likely want to import these types from the module named
//! after the async runtime you are using. For example, if you are using Tokio,
//! you should import these types from `girlboss::tokio` rather than from
//! `girlboss::common`.
//!
//! The types in this module take a [`Runtime`](crate::runtime::Runtime) as
//! their first type parameter, specifying which runtime to use. You might want
//! to use these types if you're writing a library that you also want to be
//! runtime-agnostic.

mod job;

pub use job::Job;
