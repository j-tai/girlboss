use std::borrow::Borrow;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::future::Future;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;

use crate::{Error, Job, Monitor, Result};

/// A job manager, which stores a mapping of job IDs to [`Job`]s.
///
/// The job ID type, `K`, must implement [`Ord`] because the implementation
/// currently uses a [`BTreeMap`].
pub struct Girlboss<K: Ord> {
    jobs: RwLock<BTreeMap<K, Job>>,
}

impl<K: Ord> Girlboss<K> {
    /// Creates a new empty job manager.
    pub fn new() -> Self {
        Girlboss {
            jobs: RwLock::new(BTreeMap::new()),
        }
    }

    /// Gets a job by ID.
    pub async fn get<Q>(&self, id: &Q) -> Option<Job>
    where
        Q: Ord + ?Sized,
        K: Borrow<Q>,
    {
        self.jobs.read().await.get(id).cloned()
    }

    /// Starts a new job with the provided ID.
    ///
    /// If there is already a job with the same ID, *and the job is still
    /// running (not finished)*, then this method will return `Err`. Otherwise,
    /// it returns `Ok` with a reference to the job.
    pub async fn start<F, Fut>(&self, id: impl Into<K>, func: F) -> Result<Job>
    where
        F: FnOnce(Monitor) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut jobs = self.jobs.write().await;
        match jobs.entry(id.into()) {
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
    pub async fn cleanup(&self, max_age: Duration) {
        let Some(max_time) = Instant::now().checked_sub(max_age) else {
            // The app hasn't been running for `max_age` time yet, so there's
            // nothing to delete.
            return;
        };

        let mut jobs = self.jobs.write().await;
        jobs.retain(move |_, job| match job.finished_at() {
            // If the job is finished and it's old enough, don't retain it.
            Some(finished_at) if finished_at < max_time => false,
            _ => true,
        });
    }
}

impl<K: Ord> Default for Girlboss<K> {
    fn default() -> Self {
        Girlboss::new()
    }
}
