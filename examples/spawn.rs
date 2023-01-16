extern crate async_std;

use core::time::Duration;

use pasts::{prelude::*, Executor};

async fn sleep(seconds: f64) {
    async_std::task::sleep(Duration::from_secs_f64(seconds)).await;
}

fn main() {
    let executor = Executor::default();

    // Spawn before blocking puts the task on a queue.
    executor.spawn(async {
        sleep(3.0).await;
        println!("3 seconds");
    });

    // Calling `block_on()` starting executing queued tasks.
    executor.clone().block_on(async move {
        // Spawn tasks (without being queued)
        executor.spawn(async {
            sleep(1.0).await;
            println!("1 second");
        });
        executor.spawn(async {
            sleep(2.0).await;
            println!("2 seconds");
        });

        // Finish this task before spawned tasks will complete.
        sleep(0.5).await;
        println!("½ second");
    });
}
