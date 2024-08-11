use std::future::Future;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use tokio::sync::Mutex as TokioMutex;
use tokio::task::JoinHandle;

use crate::status::AtomicJobStatus;
use crate::{JobStatus, Monitor};

/// A job, either running or finished.
///
/// This struct is only a handle for the job. Cloning a `Job` is cheap, and
/// dropping a `Job` will not cause it to stop running.
#[derive(Clone)]
pub struct Job(Arc<JobInner>);

struct JobInner {
    handle: TokioMutex<Option<JoinHandle<()>>>,
    status: AtomicJobStatus,
    started_at: Instant,
    finished_at: OnceLock<Instant>,
}

impl Job {
    /// Creates and starts a new job.
    pub fn start<F, Fut>(func: F) -> Job
    where
        F: FnOnce(Monitor) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let job = Job(Arc::new(JobInner {
            handle: TokioMutex::new(None),
            status: AtomicJobStatus::new("Starting job".into()),
            started_at: Instant::now(),
            finished_at: OnceLock::new(),
        }));
        let fut = func(Monitor(job.clone()));
        *job.0.handle.try_lock().unwrap() = Some(tokio::spawn(fut));
        job
    }

    /// Checks the latest status message reported by this job.
    pub fn status(&self) -> JobStatus {
        self.0.status.load()
    }

    /// Waits for this job to finish.
    pub async fn wait(&self) {
        if let Some(handle) = self.0.handle.lock().await.take() {
            let result = handle.await;
            if let Err(e) = result {
                self.set_status(format!("Internal error: {e}").into());
            }
        }
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
    pub fn finished_at(&self) -> Option<Instant> {
        self.0.finished_at.get().copied()
    }

    /// Returns whether this job is finished.
    pub fn is_finished(&self) -> bool {
        self.finished_at().is_some()
    }

    /// Returns the amount of wall-clock time this job has spent.
    ///
    /// If this job is finished, then this returns the time from start to
    /// finish. If this job is in progress, then this returns the time from
    /// start to now.
    pub fn elapsed(&self) -> Duration {
        self.0
            .finished_at
            .get()
            .copied()
            .unwrap_or_else(Instant::now)
            - self.0.started_at
    }
}

// Internal methods
impl Job {
    pub(crate) fn set_status(&self, status: JobStatus) {
        self.0.status.store(status);
    }
}
