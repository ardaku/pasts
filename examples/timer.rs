use async_std::task;
use core::time::Duration;

fn main() {
    pasts::Executor::new().cycle(async {
        println!("Waiting 2 seconds…");
        task::sleep(Duration::new(2, 0)).await;
        println!("Waited 2 seconds.");
        std::process::exit(0)
    });
}
