use std::borrow::Borrow;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::future::Future;
use std::time::{Duration, Instant};

use crate::common::Job;
use crate::runtime::{Runtime, Spawnable};
use crate::{Error, JobReturnStatus, Monitor, Result};

/// A job manager, which stores a mapping of job IDs to either jobs or monitors.
///
/// This job manager can store either [`Job`]s or [`Monitor`]s. You can
/// typically just import the `Girlboss` struct from the module corresponding to
/// your async runtime (for example, [`girlboss::tokio`](crate::tokio)), which
/// stores [`Job`]s of that runtime. However, if your runtime's `Job` is not
/// `Send` or `Sync`, you can choose to store [`Monitor`]s instead, which are
/// guaranteed to be `Send + Sync`.
///
/// This job manager continues to store jobs/monitors even after they are
/// finished, and this is by design. Finished jobs/monitors can be overwritten
/// with [`start`](Self::start) or cleared with [`cleanup`](Self::cleanup).
///
/// The job ID type, `K`, must implement [`Ord`] because the implementation
/// currently uses a [`BTreeMap`].
pub struct Girlboss<K: Ord, V: AsRef<Monitor> + Clone> {
    jobs: BTreeMap<K, V>,
}

impl<K: Ord, V: AsRef<Monitor> + Clone> Girlboss<K, V> {
    /// Creates a new empty job manager.
    pub fn new() -> Self {
        Girlboss {
            jobs: BTreeMap::new(),
        }
    }

    /// Gets a job or monitor by its ID.
    ///
    /// This method will continue to return jobs after they are finished. See
    /// the [struct documentation](Girlboss) for more information.
    pub fn get<Q>(&self, id: &Q) -> Option<V>
    where
        Q: Ord + ?Sized,
        K: Borrow<Q>,
    {
        self.jobs.get(id).cloned()
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

        self.jobs
            .retain(move |_, job| match job.as_ref().finished_at() {
                // If the job is finished and it's old enough, don't retain it.
                Some(finished_at) if finished_at < max_time => false,
                _ => true,
            });
    }

    fn try_insert(&mut self, id: K, f: impl FnOnce() -> V) -> Result<V> {
        match self.jobs.entry(id) {
            Entry::Vacant(vacant) => {
                let value = f();
                vacant.insert(value.clone());
                Ok(value)
            }
            Entry::Occupied(mut occupied) => {
                if occupied.get().as_ref().is_finished() {
                    let value = f();
                    occupied.insert(value.clone());
                    Ok(value)
                } else {
                    Err(Error::JobExists)
                }
            }
        }
    }
}

impl<K: Ord, R: Runtime> Girlboss<K, Job<R>> {
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
        F: FnOnce(Monitor) -> Fut,
        Fut: Spawnable<R>,
        <Fut as Future>::Output: Into<JobReturnStatus>,
    {
        self.try_insert(id.into(), || Job::start(func))
    }
}

impl<K: Ord> Girlboss<K, Monitor> {
    /// Additional implementation for a [`Monitor`]-storing job manager. See
    /// [`Girlboss<K, Job<R>>::start`] for information.
    pub fn start<R: Runtime, F, Fut>(&mut self, id: impl Into<K>, func: F) -> Result<Job<R>>
    where
        F: FnOnce(Monitor) -> Fut,
        Fut: Spawnable<R>,
        <Fut as Future>::Output: Into<JobReturnStatus>,
    {
        let mut the_job = None;
        self.try_insert(id.into(), || {
            let job = Job::start(func);
            let monitor = job.monitor().clone();
            the_job = Some(job);
            monitor
        })?;
        Ok(the_job.unwrap())
    }
}

impl<K: Ord, V: AsRef<Monitor> + Clone> Default for Girlboss<K, V> {
    fn default() -> Self {
        Girlboss::new()
    }
}
