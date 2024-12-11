//! Traits for interoperability between async runtimes.

use std::future::Future;

#[cfg(feature = "actix-rt")]
mod actix_rt;
#[cfg(feature = "tokio")]
mod tokio;

#[cfg(feature = "actix-rt")]
pub use actix_rt::ActixRt;
use sealed::sealed;
#[cfg(feature = "tokio")]
pub use tokio::Tokio;

use crate::common::Job;

/// An async runtime.
#[sealed]
pub trait Runtime: Sized {
    /// The [`JobHandle`] used by this runtime.
    type JobHandle: JobHandle<Self>;
}

/// A job handle in the runtime `R`, roughly analogous to a mutex-wrapped
/// `JoinHandle`.
#[sealed]
pub trait JobHandle<R: Runtime>: Default + 'static {
    /// Waits for the job to finish.
    fn wait(&self) -> impl std::future::Future<Output = ()>;
}

/// A future that can be spawned using the runtime `R`.
#[sealed]
pub trait Spawnable<R: Runtime>: Future + 'static {
    /// Spawns the future into the [`JobHandle`].
    fn spawn(self, handle: &R::JobHandle, job: Job<R>);
}
