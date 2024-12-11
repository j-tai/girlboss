use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;

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
