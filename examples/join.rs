#![forbid(unsafe_code)]

use pasts::prelude::*;
use async_std::task;

use std::time::Duration;

#[derive(Debug)]
struct Length(u64);

async fn timer_future(duration: u64) -> Length {
    task::sleep(Duration::new(duration, 0)).await;
    println!("Slept for {}", duration);
    Length(duration)
}

fn main() {
    pasts::spawn(|| async {
        let task = pasts::spawn(|| async {
            let one = timer_future(1);
            let two = timer_future(2);

            // This will only take two seconds, rather than
            // `(one.await, two.await)` which will take three.
            (one, two).join().await
        });
        println!("Future returned: \"{:?}\"", task.await);
    });
}
