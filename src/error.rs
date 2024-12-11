use thiserror::Error as ThisError;

/// An error type that encapsulates anything that can go wrong in this library.
#[derive(Debug, ThisError, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Returned by [`Girlboss::start`](crate::common::Girlboss::start) when the
    /// specified job ID already exists and that job is not finished.
    #[error("A job with that ID already exists")]
    JobExists,
    /// Returned by [`Job::wait`](crate::common::Job::wait) when the job
    /// returned an error or panicked.
    #[error("Job failed")]
    JobFailed,
}

/// An alias of [`Result`](std::result::Result) with the default error type
/// being [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;
