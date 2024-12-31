//! A collection of job functions to use in tests.

#![cfg_attr(not(feature = "tokio"), allow(dead_code))]

use crate::Monitor;

pub async fn instant(_: Monitor) {}

#[cfg(feature = "tokio")]
pub async fn slow(_: Monitor) {
    use std::time::Duration;

    use tokio::time::sleep;

    sleep(Duration::from_millis(100)).await;
}

#[cfg(feature = "tokio")]
pub async fn sets_status(mon: Monitor) {
    mon.report("Custom status");
    slow(mon).await;
}

#[cfg(feature = "tokio")]
pub async fn sets_status_with_write(mon: Monitor) {
    write!(mon, "trans rights");
    slow(mon).await;
}

#[cfg(feature = "tokio")]
pub async fn sets_status_with_write_fmt(mon: Monitor) {
    let number = 42;
    write!(mon, "tends to {number}");
    slow(mon).await;
}

pub async fn sets_status_by_return(_: Monitor) -> &'static str {
    "Custom status by return"
}

pub async fn fails(_: Monitor) -> Result<(), &'static str> {
    Err("oopsie")
}

pub async fn panics(_: Monitor) {
    panic!("uh oh");
}
