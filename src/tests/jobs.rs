#![cfg(feature = "tokio")]

//! A collection of job functions to use in tests.

use std::time::Duration;

use tokio::time::sleep;

use crate::common::Monitor;
use crate::runtime::Runtime;

pub async fn instant<R: Runtime>(_: Monitor<R>) {}

pub async fn slow<R: Runtime>(_: Monitor<R>) {
    sleep(Duration::from_millis(100)).await;
}

pub async fn sets_status<R: Runtime>(mon: Monitor<R>) {
    mon.report("Custom status");
    slow(mon).await;
}

pub async fn sets_status_with_write<R: Runtime>(mon: Monitor<R>) {
    write!(mon, "trans rights");
    slow(mon).await;
}

pub async fn sets_status_with_write_fmt<R: Runtime>(mon: Monitor<R>) {
    let number = 42;
    write!(mon, "tends to {number}");
    slow(mon).await;
}

pub async fn sets_status_by_return<R: Runtime>(_: Monitor<R>) -> &'static str {
    "Custom status by return"
}

pub async fn fails<R: Runtime>(_: Monitor<R>) -> Result<(), &'static str> {
    Err("oopsie")
}

pub async fn panics<R: Runtime>(_: Monitor<R>) {
    panic!("uh oh");
}
