use core::time::Duration;

use pasts::prelude::*;

include!(concat!(env!("OUT_DIR"), "/main.rs"));

struct App;

impl App {
    async fn sleep(seconds: f64) {
        async_std::task::sleep(Duration::from_secs_f64(seconds)).await;
    }

    async fn main(executor: Executor) {
        executor.spawn(async {
            Self::sleep(1.0).await;
            println!("1 second");
        });
        executor.spawn(async {
            Self::sleep(2.0).await;
            println!("2 seconds");
        });
        executor.spawn(async {
            Self::sleep(3.0).await;
            println!("3 seconds");
        });
        Self::sleep(0.5).await;
        println!("½ second");
    }
}
