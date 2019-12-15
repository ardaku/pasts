#![forbid(unsafe_code)]

use pasts::prelude::*;

mod timerfuture;

fn main() {
    let ret = pasts::CondvarInterrupt::block_on(async {
        println!("Waiting 2 seconds…");
        timerfuture::TimerFuture::new(std::time::Duration::new(2, 0)).await;
        println!("Waited 2 seconds.");
        "Complete!"
    });
    println!("Future returned: \"{}\"", ret);
}
