use core::time::Duration;

use pasts::prelude::*;

async fn sleep(seconds: f64) {
    async_std::task::sleep(Duration::from_secs_f64(seconds)).await;
}

async fn main(executor: &Executor) {
    executor.spawn(async {
        sleep(1.0).await;
        println!("1 second");
    });
    executor.spawn(async {
        sleep(2.0).await;
        println!("2 seconds");
    });
    executor.spawn(async {
        sleep(3.0).await;
        println!("3 seconds");
    });
    sleep(0.5).await;
    println!("½ second");
}
