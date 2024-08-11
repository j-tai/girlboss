use thiserror::Error as ThisError;

/// An error type that encapsulates anything that can go wrong in this library.
///
/// Currently, the only variant is [`JobExists`](Error::JobExists), though more
/// may be added in the future.
#[derive(Debug, ThisError)]
#[non_exhaustive]
pub enum Error {
    /// Returned by [`Girlboss::start`](crate::Girlboss::start) when the
    /// specified job ID already exists.
    #[error("A job with that ID already exists")]
    JobExists,
}

/// A type that represents either success or an [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
