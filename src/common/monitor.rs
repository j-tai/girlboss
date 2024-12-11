use std::fmt;

use crate::runtime::Runtime;
use crate::JobStatus;

use super::Job;

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
/// use girlboss::tokio::{Monitor, Job};
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
pub struct Monitor<R: Runtime>(pub(crate) Job<R>);

impl<R: Runtime> Monitor<R> {
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
