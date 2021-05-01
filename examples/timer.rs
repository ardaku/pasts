use async_std::task;
use core::time::Duration;

fn main() {
    pasts::block_on(async {
        println!("Waiting 2 seconds…");
        task::sleep(Duration::new(2, 0)).await;
        println!("Waited 2 seconds.");
    });
}
