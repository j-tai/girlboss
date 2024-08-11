use std::time::Duration;

use crate::tests::jobs;
use crate::{Error, Girlboss};

#[tokio::test]
async fn can_start_and_get_jobs() {
    let manager = Girlboss::<i32>::default();
    assert_eq!(manager.get(&1).await, None);
    let job1 = manager.start(1, jobs::slow).await.unwrap();
    assert_eq!(manager.get(&1).await, Some(job1.clone()));
    assert_eq!(manager.get(&2).await, None);
    let job2 = manager.start(2, jobs::slow).await.unwrap();
    assert_eq!(manager.get(&2).await, Some(job2.clone()));
    assert_ne!(job1, job2);
}

#[tokio::test]
async fn denies_duplicate_id() {
    let manager = Girlboss::<i32>::new();
    manager.start(1, jobs::slow).await.unwrap();
    let result = manager.start(1, jobs::slow).await;
    assert_eq!(result, Err(Error::JobExists))
}

#[tokio::test]
async fn replaces_finished_job() {
    let manager = Girlboss::<i32>::new();
    let job1 = manager.start(1, jobs::instant).await.unwrap();
    job1.wait().await.unwrap();
    let job1_2 = manager.start(1, jobs::instant).await.unwrap();
    assert_ne!(job1, job1_2);
}

#[tokio::test]
async fn cleanup_keeps_unfinished_jobs() {
    let manager = Girlboss::<i32>::new();
    let job1 = manager.start(1, jobs::slow).await.unwrap();
    manager.cleanup(Duration::ZERO).await;
    let job1_2 = manager.get(&1).await;
    assert_eq!(job1_2, Some(job1));
}

#[tokio::test]
async fn cleanup_removes_finished_jobs() {
    let manager = Girlboss::<i32>::new();
    let job1 = manager.start(1, jobs::instant).await.unwrap();
    job1.wait().await.unwrap();
    manager.cleanup(Duration::ZERO).await;
    let job1_2 = manager.get(&1).await;
    assert_eq!(job1_2, None);
}

#[tokio::test]
async fn cleanup_keeps_recently_finished_jobs() {
    let manager = Girlboss::<i32>::new();
    let job1 = manager.start(1, jobs::instant).await.unwrap();
    job1.wait().await.unwrap();
    manager.cleanup(Duration::from_millis(50)).await;
    manager.cleanup(Duration::MAX).await;
    let job1_2 = manager.get(&1).await;
    assert_eq!(job1_2, Some(job1));
}
