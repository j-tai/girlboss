#![cfg(feature = "tokio")]

use std::time::{Duration, Instant};

use tokio::time::sleep;

use crate::tests::jobs;
use crate::tokio::Job;
use crate::Error;

#[tokio::test]
async fn debug_impl_makes_sense() {
    let job = Job::start(jobs::instant);
    let repr = format!("{job:?}");
    assert!(repr.starts_with("Job("));
    assert!(repr.contains("0x"));
}

#[tokio::test]
async fn pointer_impl_makes_sense() {
    let job = Job::start(jobs::instant);
    let repr = format!("{job:p}");
    assert!(repr.starts_with("0x"));
}

#[tokio::test]
async fn equals_self() {
    let job = Job::start(jobs::slow);
    assert_eq!(job, job);
    assert_eq!(job, job.clone());
}

#[tokio::test]
async fn does_not_equal_other() {
    let job1 = Job::start(jobs::slow);
    let job2 = Job::start(jobs::slow);
    assert_ne!(job1, job2);
}

#[tokio::test]
async fn sets_default_status() {
    let job = Job::start(jobs::instant);
    assert_eq!(job.status().message(), "Starting job");
}

#[tokio::test]
async fn sets_custom_status() {
    let job = Job::start(jobs::sets_status);
    sleep(Duration::from_millis(50)).await;
    assert_eq!(job.status().message(), "Custom status");
}

#[tokio::test]
async fn sets_custom_status_with_write() {
    let job = Job::start(jobs::sets_status_with_write);
    sleep(Duration::from_millis(50)).await;
    assert_eq!(job.status().message(), "trans rights");
}

#[tokio::test]
async fn sets_custom_status_with_write_fmt() {
    let job = Job::start(jobs::sets_status_with_write_fmt);
    sleep(Duration::from_millis(50)).await;
    assert_eq!(job.status().message(), "tends to 42");
}

#[tokio::test]
async fn sets_custom_status_by_return_value() {
    let job = Job::start(jobs::sets_status_by_return);
    job.wait().await.unwrap();
    assert_eq!(job.status().message(), "Custom status by return");
}

#[tokio::test]
async fn outcome_is_none_when_in_progress() {
    let job = Job::start(jobs::slow);
    assert_eq!(job.outcome(), None);
    assert_eq!(job.succeeded(), false);
}

#[tokio::test]
async fn outcome_is_false_when_failed() {
    let job = Job::start(jobs::fails);
    assert_eq!(job.wait().await, Err(Error::JobFailed));
    assert_eq!(job.outcome(), Some(false));
    assert_eq!(job.succeeded(), false);
    assert_eq!(job.status().message(), "oopsie");
}

#[tokio::test]
async fn panic_is_caught() {
    let job = Job::start(jobs::panics);
    assert_eq!(job.wait().await, Err(Error::JobFailed));
    assert_eq!(job.outcome(), Some(false));
    assert_eq!(job.succeeded(), false);
    assert_eq!(job.status().message(), "The job panicked");
}

#[tokio::test]
async fn outcome_is_true_when_succeeded() {
    let job = Job::start(jobs::instant);
    job.wait().await.unwrap();
    assert_eq!(job.outcome(), Some(true));
    assert_eq!(job.succeeded(), true);
}

#[tokio::test]
async fn started_time_makes_sense() {
    let before = Instant::now();
    let job = Job::start(jobs::slow);
    let after = Instant::now();
    assert!(before <= job.monitor().started_at());
    assert!(job.monitor().started_at() <= after);
}

#[tokio::test]
async fn finished_time_makes_sense() {
    let job = Job::start(jobs::slow);
    sleep(Duration::from_millis(50)).await;
    let before = Instant::now();
    job.wait().await.unwrap();
    let after = Instant::now();
    let finished_at = job.monitor().finished_at().unwrap();
    assert!(before <= finished_at);
    assert!(finished_at <= after);
}

#[tokio::test]
async fn elapsed_time_makes_sense() {
    let job = Job::start(jobs::slow);
    job.wait().await.unwrap();
    assert!(job.monitor().elapsed() >= Duration::from_millis(100));
    assert!(job.monitor().elapsed() <= Duration::from_millis(150));
}

#[tokio::test]
async fn elapsed_time_is_retained_after_finish() {
    let job = Job::start(jobs::slow);
    job.wait().await.unwrap();
    sleep(Duration::from_millis(200)).await;
    assert!(job.monitor().elapsed() >= Duration::from_millis(100));
    assert!(job.monitor().elapsed() <= Duration::from_millis(150));
}

#[tokio::test]
async fn elapsed_time_makes_sense_before_finish() {
    let job = Job::start(jobs::slow);
    assert!(job.monitor().elapsed() <= Duration::from_millis(50));
}

#[tokio::test]
async fn is_finished_is_correct() {
    let job = Job::start(jobs::slow);
    assert_eq!(job.is_finished(), false);
    sleep(Duration::from_millis(150)).await;
    assert_eq!(job.is_finished(), true);
    job.wait().await.unwrap();
    assert_eq!(job.is_finished(), true);
}
