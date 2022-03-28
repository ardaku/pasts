//! This example shows how to create a `Past` from a future that is `!Unpin`.
//!
//! Note that this does incur an allocation, so if your environment can not
//! allocate, do not use this API.

use pasts::Task;

async fn async_main() {
    const SECOND: core::time::Duration = core::time::Duration::from_secs(1);

    let mut timer = Task::new(|| async {
        async_std::task::sleep(SECOND).await;
    });

    for _ in 0..3 {
        println!("Waiting 1 second...");
        async_std::future::poll_fn(|cx| timer.poll_next(cx)).await;
    }

    println!("Waited 3 seconds!");
}

fn main() {
    pasts::block_on(async_main());
}
