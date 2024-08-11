use std::time::{Duration, Instant};

use crate::JobStatus;

#[test]
fn creating_from_str() {
    let status = JobStatus::from("foo");
    assert_eq!(status.message(), "foo");
}

#[test]
fn creating_from_string() {
    let status = JobStatus::from("foo".to_string());
    assert_eq!(status.message(), "foo");
}

#[test]
fn timestamp_makes_sense() {
    let before = Instant::now();
    let status = JobStatus::from("test");
    let after = Instant::now();
    assert!(before <= status.timestamp());
    assert!(status.timestamp() <= after);
}

#[test]
fn age_makes_sense() {
    let status = JobStatus::from("test");
    assert!(status.age() <= Duration::from_millis(10));
}
