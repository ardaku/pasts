#![forbid(unsafe_code)]

use pasts::prelude::*;

async fn timer_future(duration: std::time::Duration) {
    pasts::spawn_blocking(move || std::thread::sleep(duration)).await
}

async fn example() -> ! {
    let every_one = || timer_future(std::time::Duration::new(1, 0));
    let every_two = || timer_future(std::time::Duration::new(2, 0));

    pasts::tasks! {
        a_fut = every_one();
        b_fut = every_two();
    };

    loop {
        pasts::select!(
            a = a_fut => {
                println!("1 Second has passed");
                a_fut.set(every_one());
            },
            b = b_fut => {
                println!("2 Seconds have passed");
                b_fut.set(every_two());
            },
        );
    }
}

fn main() {
    pasts::ThreadInterrupt::block_on(example());
}
