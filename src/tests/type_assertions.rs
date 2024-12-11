#[cfg(feature = "tokio")]
#[tokio::test]
async fn tokio() {
    use crate::tokio::Job;

    fn is_send_sync<T: Send + Sync>() {}
    fn value_is_send_sync<T: Send + Sync>(_: T) {}

    is_send_sync::<crate::tokio::Girlboss<i32>>();
    is_send_sync::<crate::tokio::Job>();
    is_send_sync::<crate::tokio::Monitor>();

    let job = Job::start(|_| async {});
    value_is_send_sync(job.wait());
}

#[cfg(feature = "actix-rt")]
#[actix_rt::test]
async fn actix_rt() {
    use crate::runtime::{ActixRt, Spawnable};

    fn value_is_spawnable<T: Spawnable<ActixRt>>(_: T) {}

    // should be able to spawn non-Send non-Sync futures
    value_is_spawnable(async {
        let raw_ptr = &() as *const _;
        // raw_ptr (!Send + !Sync) kept across await point makes this entire
        // future !Send + !Sync
        async {}.await;
        println!("{raw_ptr:?}");
    });
}
