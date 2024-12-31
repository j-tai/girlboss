use std::fmt;
use std::future::Future;
use std::sync::Arc;

use crate::runtime::{JobHandle, Runtime, Spawnable};
use crate::{Error, JobReturnStatus, JobStatus, Monitor, Result};

/// A job, either running or finished.
///
/// This struct only represents a handle for the job. Cloning a `Job` is cheap,
/// and dropping a `Job` will not cause it to stop running.
///
/// Conceptually, a `Job` is a [`Monitor`] combined with a *join handle*. This
/// join handle is async-runtime-specific and is used to [`wait`](Self::wait) on
/// the job to complete. If waiting is not needed, then you may use the job's
/// [`monitor()`](Self::monitor), which is always `Send + Sync` (whereas, if the
/// async-runtime-specific join handle is not `Send` or `Sync`, then neither
/// will the `Job`).
pub struct Job<R: Runtime> {
    handle: Arc<R::JobHandle>,
    monitor: Monitor,
}

impl<R: Runtime> Job<R> {
    /// Creates and starts a new job.
    ///
    /// The argument is the job function, which is an `async` function that
    /// takes a [`Monitor`] (for progress reporting) and returns any type that
    /// implements <code>[Into]&lt;[JobReturnStatus]&gt;</code> (for error
    /// reporting). See the [`JobReturnStatus` documentation](JobReturnStatus)
    /// for the complete list of types that the function may return.
    ///
    /// # Examples
    ///
    /// Starting a new job:
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() {
    /// use girlboss::tokio::Job;
    ///
    /// let job = Job::start(|mon| async move {
    ///     // ... long running task goes here ...
    ///     write!(mon, "I'm done!");
    /// });
    /// job.wait().await.unwrap();
    /// assert_eq!(job.status().message(), "I'm done!");
    /// # }
    /// ```
    pub fn start<F, Fut>(func: F) -> Self
    where
        F: FnOnce(Monitor) -> Fut,
        Fut: Spawnable<R>,
        <Fut as Future>::Output: Into<JobReturnStatus>,
    {
        let job = Job {
            handle: Arc::new(R::JobHandle::default()),
            monitor: Monitor::starting(),
        };

        let fut = func(job.monitor.clone());
        fut.spawn(&job.handle, job.monitor.clone());
        job
    }

    /// Waits for this job to finish.
    ///
    /// If the job indicated that it failed, this returns
    /// <code>Err([Error::JobFailed])</code>. Otherwise, it returns `Ok(())`.
    ///
    /// If the job is already finished, then this method does nothing other than
    /// return `Ok` or `Err` as described above.
    pub async fn wait(&self) -> Result<()> {
        self.handle.wait().await;
        if self.monitor.succeeded() {
            Ok(())
        } else {
            Err(Error::JobFailed)
        }
    }
}

// Aliases for the job's monitor
impl<R: Runtime> Job<R> {
    /// Returns a reference to this job's [`Monitor`]. The monitor can be used
    /// to check for the job status, among other things.
    pub fn monitor(&self) -> &Monitor {
        &self.monitor
    }

    /// Alias of
    /// <code>self.[monitor](Self::monitor)().[status](Monitor::status)</code>.
    pub fn status(&self) -> JobStatus {
        self.monitor.status()
    }

    /// Alias of
    /// <code>self.[monitor](Self::monitor)().[outcome](Monitor::outcome)</code>.
    pub fn outcome(&self) -> Option<bool> {
        self.monitor.outcome()
    }

    /// Alias of
    /// <code>self.[monitor](Self::monitor)().[is_finished](Monitor::is_finished)</code>.
    pub fn is_finished(&self) -> bool {
        self.monitor.is_finished()
    }

    /// Alias of
    /// <code>self.[monitor](Self::monitor)().[succeeded](Monitor::succeeded)</code>.
    pub fn succeeded(&self) -> bool {
        self.monitor.succeeded()
    }
}

impl<R: Runtime> Clone for Job<R> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            monitor: self.monitor.clone(),
        }
    }
}

impl<R: Runtime> PartialEq for Job<R> {
    fn eq(&self, other: &Self) -> bool {
        self.monitor == other.monitor
    }
}

impl<R: Runtime> Eq for Job<R> {}

impl<R: Runtime> fmt::Debug for Job<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Job").field(&self.monitor).finish()
    }
}

impl<R: Runtime> fmt::Pointer for Job<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.monitor.fmt(f)
    }
}
