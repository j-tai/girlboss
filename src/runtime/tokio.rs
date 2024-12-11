use std::future::Future;
use std::panic::AssertUnwindSafe;

use futures::FutureExt;
use sealed::sealed;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::common::Job;
use crate::JobReturnStatus;

/// Represents the Tokio async runtime.
pub enum Tokio {}

#[derive(Default)]
pub struct TokioHandle(Mutex<Option<JoinHandle<()>>>);

#[sealed]
impl super::Runtime for Tokio {
    type JobHandle = TokioHandle;
}

#[sealed]
impl super::JobHandle<Tokio> for TokioHandle {
    async fn wait(&self) {
        if let Some(handle) = self.0.lock().await.take() {
            // If the task got cancelled for some reason, don't worry about it.
            // Also, the task shouldn't panic because we `catch_unwind`.
            let _ = handle.await;
        }
    }
}

#[sealed]
impl<F> super::Spawnable<Tokio> for F
where
    F: Future + Send + 'static,
    F::Output: Into<JobReturnStatus>,
{
    fn spawn(self, handle: &TokioHandle, job: Job<Tokio>) {
        *handle.0.try_lock().unwrap() = Some(tokio::task::spawn(async move {
            let result = AssertUnwindSafe(self).catch_unwind().await;
            job.set_finished(result);
        }));
    }
}
