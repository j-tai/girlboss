use std::cell::RefCell;
use std::future::Future;
use std::panic::AssertUnwindSafe;

use actix_rt::task::JoinHandle;
use futures::FutureExt;
use sealed::sealed;

use crate::common::Job;
use crate::JobReturnStatus;

/// Represents the actix-rt async runtime.
pub enum ActixRt {}

#[derive(Default)]
pub struct ActixRtHandle(RefCell<Option<JoinHandle<()>>>);

#[sealed]
impl super::Runtime for ActixRt {
    type JobHandle = ActixRtHandle;
}

#[sealed]
impl super::JobHandle<ActixRt> for ActixRtHandle {
    async fn wait(&self) {
        if let Some(handle) = &mut *self.0.borrow_mut() {
            let _ = handle.await;
        }
    }
}

#[sealed]
impl<F> super::Spawnable<ActixRt> for F
where
    F: Future + 'static,
    F::Output: Into<JobReturnStatus>,
{
    fn spawn(self, handle: &ActixRtHandle, job: Job<ActixRt>) {
        *handle.0.borrow_mut() = Some(actix_rt::spawn(async move {
            let result = AssertUnwindSafe(self).catch_unwind().await;
            job.set_finished(result);
        }));
    }
}