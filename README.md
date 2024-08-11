# Girlboss

Girlboss is a simple async job manager with progress tracking.

Girlboss allows you to start background tasks and monitor their progress. You can choose to keep track of the jobs yourself, or use the provided manager to look up jobs by whatever ID you want.

For example, this can be useful in web servers where a user starts a job and periodically checks its status.

## Job manager

A `Girlboss` instance manages a set of jobs, allowing you to look them up by ID:

```rust
use std::time::Duration;
use girlboss::{Girlboss, Monitor};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    // Create a manager with strings as job IDs:
    let manager: Girlboss<String> = Girlboss::new();

    // Start a job:
    let job = manager.start("myJobId", long_running_task).await.unwrap();

    // Look up the job by ID:
    let job = manager.get("myJobId").await.expect("job not found");

    // Check the status of the job:
    sleep(Duration::from_millis(50)).await;
    assert_eq!(job.status().message(), "Computing the meaning of life...");

    // Wait for the job to complete:
    job.wait().await.expect("job failed");
    assert_eq!(job.status().message(), "The meaning of life is 42");

    // See how long the job took:
    assert!(job.elapsed() >= Duration::from_millis(200));
}

/// A long running task that we want to run in the background.
async fn long_running_task(mon: Monitor) {
    write!(mon, "Computing the meaning of life...");

    sleep(Duration::from_millis(200)).await;
    let meaning = 42;

    write!(mon, "The meaning of life is {meaning}");
}
```

## Jobs without a manager

Alternatively, if you don't need a manager, you can start jobs directly and take advantage of progress tracking:

```rust
use std::time::Duration;
use girlboss::{Job, Monitor};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    // Start a job:
    let job = Job::start(long_running_task);

    // Check the status of the job:
    sleep(Duration::from_millis(50)).await;
    assert_eq!(job.status().message(), "Computing the meaning of life...");

    // Wait for the job to complete:
    job.wait().await.expect("job failed");
    assert_eq!(job.status().message(), "The meaning of life is 42");

    // See how long the job took:
    assert!(job.elapsed() >= Duration::from_millis(200));
}

/// A long running task that we want to run in the background.
async fn long_running_task(mon: Monitor) {
    write!(mon, "Computing the meaning of life...");

    sleep(Duration::from_millis(200)).await;
    let meaning = 42;

    write!(mon, "The meaning of life is {meaning}");
}
```

## Error handling

Jobs can optionally return an error, which is then reported through the status:

```rust
use girlboss::{Job, Monitor};

#[tokio::main]
async fn main() {
    // Start a job:
    let job = Job::start(long_running_task);

    // Wait for the job to complete:
    job.wait().await.unwrap_err();
    assert_eq!(job.status().message(), "Error: The meaning of life could not be found");
    assert_eq!(job.succeeded(), false);
}

/// A long running task that we want to run in the background.
async fn long_running_task(mon: Monitor) -> Result<(), String> {
    write!(mon, "Computing the meaning of life...");
    Err("The meaning of life could not be found".to_string())
}
```

## License

[MIT](LICENSE)
