use std::fmt;
use std::future::Future;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use crate::runtime::{JobHandle, Runtime, Spawnable};
use crate::status::AtomicJobStatus;
use crate::{Error, JobReturnStatus, JobStatus, Result};

use super::Monitor;

/// A job, either running or finished.
///
/// This struct only represents a handle for the job. Cloning a `Job` is cheap,
/// and dropping a `Job` will not cause it to stop running.
pub struct Job<R: Runtime>(Arc<JobInner<R>>);

struct JobInner<R: Runtime> {
    handle: R::JobHandle,
    status: AtomicJobStatus,
    started_at: Instant,
    finished: OnceLock<JobFinishedInfo>,
}

#[derive(Debug)]
struct JobFinishedInfo {
    finished_at: Instant,
    is_success: bool,
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
        F: FnOnce(Monitor<R>) -> Fut,
        Fut: Spawnable<R>,
        <Fut as Future>::Output: Into<JobReturnStatus>,
    {
        let job = Job(Arc::new(JobInner {
            handle: R::JobHandle::default(),
            status: AtomicJobStatus::new("Starting job".into()),
            started_at: Instant::now(),
            finished: OnceLock::new(),
        }));

        let fut = func(Monitor(job.clone()));
        fut.spawn(&job.0.handle, job.clone());
        job
    }

    /// Returns the latest status message reported by this job.
    pub fn status(&self) -> JobStatus {
        self.0.status.load()
    }

    /// Waits for this job to finish.
    ///
    /// If the job indicated that it failed, this returns
    /// <code>Err([Error::JobFailed])</code>. Otherwise, it returns `Ok(())`.
    ///
    /// If the job is already finished, then this method does nothing other than
    /// return `Ok` or `Err` as described above.
    pub async fn wait(&self) -> Result<()> {
        self.0.handle.wait().await;
        if self.0.finished.get().unwrap().is_success {
            Ok(())
        } else {
            Err(Error::JobFailed)
        }
    }

    /// Returns whether this job finished successfully, or `None` if it is still
    /// in progress.
    ///
    /// Whether the job is considered successful or not is determined by the
    /// job's return value. See [`JobReturnStatus`] for the allowed types of the
    /// return value and which ones correspond to success or failure.
    ///
    /// This method is guaranteed to return `Some(_)` if and only if
    /// [`self.is_finished()`](Self::is_finished) returns `true` (barring the
    /// fact that the job could have changed from "in progress" to "finished" in
    /// between two method calls).
    pub fn outcome(&self) -> Option<bool> {
        self.0.finished.get().map(|info| info.is_success)
    }

    /// Returns whether this job is finished.
    ///
    /// Equivalent to `self.outcome().is_some()`.
    pub fn is_finished(&self) -> bool {
        self.outcome().is_some()
    }

    /// Returns `true` if this job finished successfully.
    ///
    /// If this job is still in progress, then this returns `false`. See
    /// [`outcome`](Self::outcome) for more information about "successful" and
    /// "failed" jobs.
    ///
    /// Equivalent to `self.outcome().unwrap_or(false)`.
    pub fn succeeded(&self) -> bool {
        self.outcome().unwrap_or(false)
    }

    /// Returns the [`Instant`] that this job was started.
    pub fn started_at(&self) -> Instant {
        self.0.started_at
    }

    /// Returns the [`Instant`] that this job finished, or [`None`] if it is
    /// still in progress.
    ///
    /// Note that the time finished will be recorded correctly even if the job
    /// is never [`wait`](Job::wait)ed on. That is, this method returns the time
    /// that the job finished, not when the job was found to be finished by
    /// `wait`.
    ///
    /// This method is guaranteed to return `Some(_)` if and only if
    /// [`self.outcome()`](Self::outcome) returns `Some(_)` (barring the fact
    /// that the job could have changed from "in progress" to "finished" in
    /// between two method calls).
    pub fn finished_at(&self) -> Option<Instant> {
        self.0.finished.get().map(|info| info.finished_at)
    }

    /// Returns the amount of wall-clock time this job has spent.
    ///
    /// If this job is finished, then this returns the time from start to
    /// finish. If this job is in progress, then this returns the time from
    /// start to now.
    pub fn elapsed(&self) -> Duration {
        self.finished_at().unwrap_or_else(Instant::now) - self.0.started_at
    }
}

// Internal methods
impl<R: Runtime> Job<R> {
    pub(crate) fn set_status(&self, status: JobStatus) {
        self.0.status.store(status);
    }

    pub(crate) fn set_finished<T, E>(&self, result: Result<T, E>)
    where
        T: Into<JobReturnStatus>,
    {
        // Did it panic?
        let mut return_status = match result {
            Ok(output) => output.into(),
            Err(_) => JobReturnStatus::panicked(),
        };

        // Write the final message
        if let Some(final_message) = return_status.message.take() {
            self.set_status(final_message.into());
        }

        // Record the job completion
        let finished_info = JobFinishedInfo {
            finished_at: Instant::now(),
            is_success: return_status.is_success,
        };
        self.0.finished.set(finished_info).unwrap();
    }
}

impl<R: Runtime> Clone for Job<R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<R: Runtime> PartialEq for Job<R> {
    fn eq(&self, other: &Self) -> bool {
        Arc::as_ptr(&self.0) == Arc::as_ptr(&other.0)
    }
}

impl<R: Runtime> Eq for Job<R> {}

impl<R: Runtime> fmt::Debug for Job<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Job").field(&Arc::as_ptr(&self.0)).finish()
    }
}

impl<R: Runtime> fmt::Pointer for Job<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.0, f)
    }
}
