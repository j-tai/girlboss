#![cfg(feature = "actix-rt")]

use crate::actix_rt::Job;
use crate::tests::jobs;
use crate::Error;

#[actix_rt::test]
async fn sets_custom_status_by_return_value() {
    let job = Job::start(jobs::sets_status_by_return);
    job.wait().await.unwrap();
    assert_eq!(job.status().message(), "Custom status by return");
}

#[actix_rt::test]
async fn panic_is_caught() {
    let job = Job::start(jobs::panics);
    assert_eq!(job.wait().await, Err(Error::JobFailed));
    assert_eq!(job.outcome(), Some(false));
    assert_eq!(job.succeeded(), false);
    assert_eq!(job.status().message(), "The job panicked");
}
