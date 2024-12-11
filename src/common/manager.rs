use std::borrow::Borrow;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::future::Future;
use std::time::{Duration, Instant};

use crate::runtime::{Runtime, Spawnable};
use crate::{Error, JobReturnStatus, Result};

use super::{Job, Monitor};

/// A job manager, which stores a mapping of job IDs to [`Job`]s.
///
/// This job manager continues to store jobs even after they are finished, and
/// this is by design. Finished jobs can be overwritten with
/// [`start`](Self::start) or cleared with [`cleanup`](Self::cleanup).
///
/// The job ID type, `K`, must implement [`Ord`] because the implementation
/// currently uses a [`BTreeMap`].
pub struct Girlboss<R: Runtime, K: Ord> {
    jobs: BTreeMap<K, Job<R>>,
}

impl<R: Runtime, K: Ord> Girlboss<R, K> {
    /// Creates a new empty job manager.
    pub fn new() -> Self {
        Girlboss {
            jobs: BTreeMap::new(),
        }
    }

    /// Gets a job by ID.
    ///
    /// This method will continue to return jobs after they are finished. See
    /// the [struct documentation](Girlboss) for more information.
    pub fn get<Q>(&self, id: &Q) -> Option<Job<R>>
    where
        Q: Ord + ?Sized,
        K: Borrow<Q>,
    {
        self.jobs.get(id).cloned()
    }

    /// Starts and returns a new job with the provided ID.
    ///
    /// If there is already a job with the same ID, then:
    ///
    /// * If the old job is **finished**, then the old job will be **overwritten**
    ///   with the new job.
    /// * If the old job is **not** finished, then the new job will **not** be
    ///   started and this method will return
    ///   <code>Err([Error::JobExists])</code>.
    ///
    /// See [`Job::start`] for information about the job function.
    pub fn start<F, Fut>(&mut self, id: impl Into<K>, func: F) -> Result<Job<R>>
    where
        F: FnOnce(Monitor<R>) -> Fut,
        Fut: Spawnable<R>,
        <Fut as Future>::Output: Into<JobReturnStatus>,
    {
        match self.jobs.entry(id.into()) {
            Entry::Vacant(vacant) => {
                let job = Job::start(func);
                vacant.insert(job.clone());
                Ok(job)
            }
            Entry::Occupied(mut occupied) => {
                if occupied.get().is_finished() {
                    let job = Job::start(func);
                    occupied.insert(job.clone());
                    Ok(job)
                } else {
                    Err(Error::JobExists)
                }
            }
        }
    }

    /// Removes all jobs that finished at least `max_age` time ago.
    ///
    /// If `max_age` is [`Duration::ZERO`], then all finished jobs are removed.
    ///
    /// Jobs that are still in progress are never touched.
    pub fn cleanup(&mut self, max_age: Duration) {
        let Some(max_time) = Instant::now().checked_sub(max_age) else {
            // The app hasn't been running for `max_age` time yet, so there's
            // nothing to delete.
            return;
        };

        self.jobs.retain(move |_, job| match job.finished_at() {
            // If the job is finished and it's old enough, don't retain it.
            Some(finished_at) if finished_at < max_time => false,
            _ => true,
        });
    }
}

impl<R: Runtime, K: Ord> Default for Girlboss<R, K> {
    fn default() -> Self {
        Girlboss::new()
    }
}
