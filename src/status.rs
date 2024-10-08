use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;

use crate::Job;

/// A status message reported from a job.
#[derive(Clone)]
pub struct JobStatus(Arc<JobStatusInner>);

struct JobStatusInner {
    message: Cow<'static, str>,
    timestamp: Instant,
}

impl JobStatus {
    /// The reported message.
    pub fn message(&self) -> &str {
        &self.0.message
    }

    /// The timestamp of the report.
    pub fn timestamp(&self) -> Instant {
        self.0.timestamp
    }

    /// The time ago that this was reported.
    pub fn age(&self) -> Duration {
        Instant::now() - self.timestamp()
    }
}

impl<T: Into<Cow<'static, str>>> From<T> for JobStatus {
    fn from(value: T) -> Self {
        JobStatus(Arc::new(JobStatusInner {
            message: value.into(),
            timestamp: Instant::now(),
        }))
    }
}

impl fmt::Debug for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JobStatus")
            .field("message", &&self.0.message[..])
            .field("timestamp", &self.0.timestamp)
            .finish()
    }
}

pub(crate) struct AtomicJobStatus(ArcSwap<JobStatusInner>);

impl AtomicJobStatus {
    pub fn new(status: JobStatus) -> Self {
        AtomicJobStatus(ArcSwap::new(status.0))
    }

    pub fn load(&self) -> JobStatus {
        JobStatus(self.0.load().clone())
    }

    pub fn store(&self, status: JobStatus) {
        self.0.store(status.0);
    }
}

/// An object used by a running job to report its progress.
///
/// Jobs can use the [`report`](Monitor::report) method or the standard
/// library's [`write!`] macro to report messages.
///
/// # Examples
///
/// Reporting progress:
///
/// ```
/// # #[tokio::main]
/// # async fn main() {
/// use girlboss::{Monitor, Job};
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
#[derive(Clone, Debug)]
pub struct Monitor(pub(crate) Job);

impl Monitor {
    /// Reports a new status message.
    ///
    /// If your message is already a [`String`] and you are able to give
    /// ownership of the message, then this method avoids an allocation compared
    /// to using [`write!`]. However, if your message is a `&str` or needs to be
    /// [`format`]ted, then you should use [`write!`].
    pub fn report(&self, status: impl Into<JobStatus>) {
        self.0.set_status(status.into());
    }

    /// Implementation to allow use with [`write!`].
    pub fn write_fmt(&self, args: fmt::Arguments<'_>) {
        match args.as_str() {
            Some(s) => self.report(s),
            None => self.report(args.to_string()),
        }
    }
}
