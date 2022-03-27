//! This example shows how to create a `Past` from a future that is `!Unpin`.
//!
//! Note that this does incur an allocation, so if your environment can not
//! allocate, do not use this API.

use pasts::Past;

async fn async_main() {
    const SECOND: core::time::Duration = core::time::Duration::from_secs(1);

    let mut timer = Past::pin(|| async {
        async_std::task::sleep(SECOND).await;
    });
    
    for _ in 0..3 {
        println!("Waiting 1 second...");
        timer.next().await;
    }

    println!("Waited 3 seconds!");
}

fn main() {
    pasts::block_on(async_main());
}
