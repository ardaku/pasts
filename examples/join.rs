#![forbid(unsafe_code)]

use pasts::prelude::*;
use pasts::CvarExec;

#[derive(Debug)]
struct Length(u64);

async fn timer_future(duration: u64) -> Length {
    pasts::spawn_blocking(move || {
        std::thread::sleep(std::time::Duration::new(duration, 0));
        println!("Slept for {}", duration);
        Length(duration)
    })
    .await
}

fn main() {
    static EXECUTOR: CvarExec = CvarExec::new();
    let ret = EXECUTOR.block_on(async {
        let one = timer_future(1);
        let two = timer_future(2);

        // This will only take two seconds, rather than `(one.await, two.await)`
        // which will take three.
        (one, two).join().await
    });
    println!("Future returned: \"{:?}\"", ret);
}
