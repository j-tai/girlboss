use std::fmt;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use futures::FutureExt;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::status::AtomicJobStatus;
use crate::{Error, JobOutput, JobStatus, Monitor, Result};

/// A job, either running or finished.
///
/// This struct only represents a handle for the job. Cloning a `Job` is cheap,
/// and dropping a `Job` will not cause it to stop running.
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
    /// The argument is the job function, which is an `async` function that
    /// takes a [`Monitor`] (for progress reporting) and returns any type that
    /// implements [`JobOutput`] (for error reporting). See [`JobOutput`] for
    /// the complete list of types that the function may return.
    ///
    /// # Examples
    ///
    /// Starting a new job:
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() {
    /// use girlboss::Job;
    ///
    /// let job = Job::start(|mon| async move {
    ///     // ... long running task goes here ...
    ///     write!(mon, "I'm done!");
    /// });
    /// job.wait().await.unwrap();
    /// assert_eq!(job.status().message(), "I'm done!");
    /// # }
    /// ```
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
            // If the job panics, we still want to clean up the job.
            // AssertUnwindSafe should be fine here, since whatever the future
            // does is the user's responsibility, and we don't share any state
            // with `fut`.
            let result = AssertUnwindSafe(fut).catch_unwind().await;

            // Did it panic?
            let is_success = match result {
                Ok(output) => {
                    let success = output.is_success();
                    if let Some(message) = output.into_message() {
                        job2.set_status(message.into());
                    }
                    success
                }
                Err(_error) => {
                    // There's not much I can do to make a Box<dyn Any> human
                    // readable...
                    job2.set_status("The job panicked".into());
                    false
                    // Hopefully dropping the error object doesn't panic,
                    // otherwise God help us
                }
            };

            // Record the job completion
            let finished_info = JobFinishedInfo {
                finished_at: Instant::now(),
                is_success,
            };
            job2.0.finished.set(finished_info).unwrap();
        });

        // This `unwrap` doesn't panic because no one else has access to the
        // handle mutex. The job function has access to a `Monitor`, but that
        // cannot be used to gain access to the `Job` instance and touch the
        // handler via `wait()`.
        *job.0.handle.try_lock().unwrap() = Some(handle);
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
        if let Some(handle) = self.0.handle.lock().await.take() {
            // If the task got cancelled for some reason, don't worry about it.
            // Also, the task shouldn't panic because we `catch_unwind`.
            let _ = handle.await;
        }
        if self.0.finished.get().unwrap().is_success {
            Ok(())
        } else {
            Err(Error::JobFailed)
        }
    }

    /// Returns true if this job finished successfully.
    ///
    /// Whether the job is considered successful or not is determined by the
    /// job's return value. See [`JobOutput`] for the allowed types of the
    /// return value and which ones correspond to success or failure.
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

impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        Arc::as_ptr(&self.0) == Arc::as_ptr(&other.0)
    }
}

impl Eq for Job {}

impl fmt::Debug for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Job").field(&Arc::as_ptr(&self.0)).finish()
    }
}

impl fmt::Pointer for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.0, f)
    }
}
