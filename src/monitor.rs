use std::fmt;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use crate::status::AtomicJobStatus;
use crate::{JobReturnStatus, JobStatus};

/// Stores progress data for a job.
///
/// Jobs can use the [`report`](Monitor::report) method or the standard
/// library's [`write!`] macro to report messages. In addition, a `Monitor` can
/// be used to query the current [`status`](Self::status) of the job, check when
/// it [`started_at`](Self::started_at) or [`finished_at`](Self::finished_at),
/// and so on.
///
/// Cloning a `Monitor` is cheap. The cloned `Monitor` represents the same
/// monitor as the original, so both can be used to query the same job's status.
///
/// # Examples
///
/// Reporting progress:
///
/// ```
/// # #[tokio::main]
/// # async fn main() {
/// use girlboss::Monitor;
/// use girlboss::tokio::Job;
///
/// async fn long_running_task(mon: Monitor) {
///     write!(mon, "Starting task."); // alternatively: mon.report("Starting task.");
///     let meaning = 42;
///     write!(mon, "The meaning of life is {meaning}");
/// }
///
/// let job = Job::start(long_running_task);
/// job.wait().await.unwrap();
/// assert_eq!(job.status().message(), "The meaning of life is 42");
/// # }
/// ```
#[derive(Clone)]
pub struct Monitor(Arc<MonitorInner>);

struct MonitorInner {
    status: AtomicJobStatus,
    started_at: Instant,
    finished: OnceLock<JobFinishedInfo>,
}

#[derive(Debug)]
struct JobFinishedInfo {
    finished_at: Instant,
    is_success: bool,
}

impl Monitor {
    /// Returns the latest status message reported to this `Monitor`.
    pub fn status(&self) -> JobStatus {
        self.0.status.load()
    }

    /// Reports a new status message to this `Monitor`.
    ///
    /// If your message is already a [`String`] and you are able to give
    /// ownership of the message, then this method avoids an allocation compared
    /// to using [`write!`]. However, if your message is a `&str` or needs to be
    /// [`format`]ted, then you should use [`write!`].
    pub fn report(&self, status: impl Into<JobStatus>) {
        self.0.status.store(status.into());
    }

    /// Implementation to allow use with [`write!`].
    pub fn write_fmt(&self, args: fmt::Arguments<'_>) {
        match args.as_str() {
            Some(s) => self.report(s),
            None => self.report(args.to_string()),
        }
    }

    /// Returns whether the job finished successfully, or `None` if it is still
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

    /// Returns whether the job is finished.
    ///
    /// Equivalent to `self.outcome().is_some()`.
    pub fn is_finished(&self) -> bool {
        self.outcome().is_some()
    }

    /// Returns `true` if the job finished successfully.
    ///
    /// If the job is still in progress, then this returns `false`. See
    /// [`outcome`](Self::outcome) for more information about "successful" and
    /// "failed" jobs.
    ///
    /// Equivalent to `self.outcome().unwrap_or(false)`.
    pub fn succeeded(&self) -> bool {
        self.outcome().unwrap_or(false)
    }

    /// Returns the [`Instant`] that the job was started.
    pub fn started_at(&self) -> Instant {
        self.0.started_at
    }

    /// Returns the [`Instant`] that the job finished, or [`None`] if it is
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

    /// Returns the amount of wall-clock time the job has spent.
    ///
    /// If the job is finished, then this returns the time from start to
    /// finish. If the job is in progress, then this returns the time from
    /// start to now.
    pub fn elapsed(&self) -> Duration {
        self.finished_at().unwrap_or_else(Instant::now) - self.0.started_at
    }
}

// Internal methods
impl Monitor {
    pub(crate) fn starting() -> Monitor {
        Monitor(Arc::new(MonitorInner {
            status: AtomicJobStatus::new("Starting job".into()),
            started_at: Instant::now(),
            finished: OnceLock::new(),
        }))
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
            self.report(final_message);
        }

        // Record the job completion
        let finished_info = JobFinishedInfo {
            finished_at: Instant::now(),
            is_success: return_status.is_success,
        };
        self.0.finished.set(finished_info).unwrap();
    }
}

impl PartialEq for Monitor {
    fn eq(&self, other: &Self) -> bool {
        Arc::as_ptr(&self.0) == Arc::as_ptr(&other.0)
    }
}

impl Eq for Monitor {}

impl fmt::Debug for Monitor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Monitor")
            .field(&Arc::as_ptr(&self.0))
            .finish()
    }
}

impl fmt::Pointer for Monitor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Arc::as_ptr(&self.0).fmt(f)
    }
}
