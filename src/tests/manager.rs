#![cfg(feature = "tokio")]

use std::time::Duration;

use crate::runtime::Tokio;
use crate::tests::jobs;
use crate::tokio::Girlboss;
use crate::{Error, Monitor};

#[tokio::test]
async fn can_start_and_get_jobs() {
    let mut manager = Girlboss::<i32>::default();
    assert_eq!(manager.get(&1), None);
    let job1 = manager.start(1, jobs::slow).unwrap();
    assert_eq!(manager.get(&1), Some(job1.clone()));
    assert_eq!(manager.get(&2), None);
    let job2 = manager.start(2, jobs::slow).unwrap();
    assert_eq!(manager.get(&2), Some(job2.clone()));
    assert_ne!(job1, job2);
}

#[tokio::test]
async fn denies_duplicate_id() {
    let mut manager = Girlboss::<i32>::new();
    manager.start(1, jobs::slow).unwrap();
    let result = manager.start(1, jobs::slow);
    assert_eq!(result, Err(Error::JobExists))
}

#[tokio::test]
async fn replaces_finished_job() {
    let mut manager = Girlboss::<i32>::new();
    let job1 = manager.start(1, jobs::instant).unwrap();
    job1.wait().await.unwrap();
    let job1_2 = manager.start(1, jobs::instant).unwrap();
    assert_ne!(job1, job1_2);
}

#[tokio::test]
async fn cleanup_keeps_unfinished_jobs() {
    let mut manager = Girlboss::<i32>::new();
    let job1 = manager.start(1, jobs::slow).unwrap();
    manager.cleanup(Duration::ZERO);
    let job1_2 = manager.get(&1);
    assert_eq!(job1_2, Some(job1));
}

#[tokio::test]
async fn cleanup_removes_finished_jobs() {
    let mut manager = Girlboss::<i32>::new();
    let job1 = manager.start(1, jobs::instant).unwrap();
    job1.wait().await.unwrap();
    manager.cleanup(Duration::ZERO);
    let job1_2 = manager.get(&1);
    assert_eq!(job1_2, None);
}

#[tokio::test]
async fn cleanup_keeps_recently_finished_jobs() {
    let mut manager = Girlboss::<i32>::new();
    let job1 = manager.start(1, jobs::instant).unwrap();
    job1.wait().await.unwrap();
    manager.cleanup(Duration::from_millis(50));
    manager.cleanup(Duration::MAX);
    let job1_2 = manager.get(&1);
    assert_eq!(job1_2, Some(job1));
}

#[tokio::test]
async fn store_monitors() {
    let mut manager = crate::Girlboss::<i32, Monitor>::new();
    let job1 = manager.start::<Tokio, _, _>(1, jobs::instant).unwrap();
    let mon1 = manager.get(&1).unwrap();
    assert_eq!(*job1.monitor(), mon1);

    let job2 = manager.start::<Tokio, _, _>(2, jobs::slow).unwrap();
    let mon2 = manager.get(&2).unwrap();
    let mon2_1 = manager.get(&2).unwrap();
    assert_eq!(*job2.monitor(), mon2);
    assert_eq!(*job2.monitor(), mon2_1);
    assert_eq!(mon2, mon2_1);

    assert_ne!(mon1, mon2);
}
