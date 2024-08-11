use std::future::Future;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::status::AtomicJobStatus;
use crate::{Error, JobOutput, JobStatus, Monitor, Result};

/// A job, either running or finished.
///
/// This struct is only a handle for the job. Cloning a `Job` is cheap, and
/// dropping a `Job` will not cause it to stop running.
#[derive(Clone)]
pub struct Job(Arc<JobInner>);

struct JobInner {
    handle: Mutex<Option<JoinHandle<()>>>,
    status: AtomicJobStatus,
    started_at: Instant,
    finished: OnceLock<JobFinishedInfo>,
}

#[derive(Debug)]
struct JobFinishedInfo {
    finished_at: Instant,
    is_success: bool,
}

impl Job {
    /// Creates and starts a new job.
    ///
    /// The argument is the job function, which takes a [`Monitor`] (for
    /// progress reporting) and returns any type that implements [`JobOutput`]
    /// (for error reporting). See [`JobOutput`] for the complete list of types
    /// that the function may return.
    pub fn start<F, Fut>(func: F) -> Job
    where
        F: FnOnce(Monitor) -> Fut,
        Fut: Future + Send + 'static,
        <Fut as Future>::Output: JobOutput,
    {
        let job = Job(Arc::new(JobInner {
            handle: Mutex::new(None),
            status: AtomicJobStatus::new("Starting job".into()),
            started_at: Instant::now(),
            finished: OnceLock::new(),
        }));

        let fut = func(Monitor(job.clone()));
        let job2 = job.clone();
        let handle = tokio::spawn(async move {
            let output = fut.await;
            let finished_info = JobFinishedInfo {
                is_success: output.is_success(),
                finished_at: Instant::now(),
            };
            if let Some(message) = output.into_message() {
                job2.set_status(message.into());
            }
            job2.0.finished.set(finished_info).unwrap();
        });

        // This `unwrap` doesn't panic because no one else has access to the
        // handle mutex. The job function has access to a `Monitor`, but that
        // cannot be used to gain access to the `Job` instance and touch the
        // handler via `wait()`.
        *job.0.handle.try_lock().unwrap() = Some(handle);
        job
    }

    /// Checks the latest status message reported by this job.
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
        if let Some(handle) = self.0.handle.lock().await.take() {
            let result = handle.await;
            if let Err(e) = result {
                self.set_status(format!("Internal error: {e}").into());
            }
        }
        if self.0.finished.get().unwrap().is_success {
            Ok(())
        } else {
            Err(Error::JobFailed)
        }
    }

    /// Returns true if this job finished successfully.
    ///
    /// If this job is still in progress, then this returns `false`.
    pub fn succeeded(&self) -> bool {
        self.0
            .finished
            .get()
            .map(|info| info.is_success)
            .unwrap_or(false)
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
        self.0.finished.get().map(|info| info.finished_at)
    }

    /// Returns whether this job is finished.
    pub fn is_finished(&self) -> bool {
        self.0.finished.get().is_some()
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
impl Job {
    pub(crate) fn set_status(&self, status: JobStatus) {
        self.0.status.store(status);
    }
}
