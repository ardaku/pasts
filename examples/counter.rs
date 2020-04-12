#![forbid(unsafe_code)]

use pasts::Select;
use std::future::Future;
use std::cell::RefCell;

async fn timer_future(duration: std::time::Duration) {
    pasts::spawn_blocking(move || std::thread::sleep(duration)).await
}

async fn one(state: &RefCell<usize>) {
    println!("Starting task one");
    while *state.borrow() < 5 {
        timer_future(std::time::Duration::new(1, 0)).await;
        let mut state = state.borrow_mut();
        println!("One {}", *state);
        *state += 1;
    }
    println!("Finish task one");
}

async fn two(state: &RefCell<usize>) {
    println!("Starting task two");
    loop {
        timer_future(std::time::Duration::new(2, 0)).await;
        let mut state = state.borrow_mut();
        println!("Two {}", *state);
        *state += 1;
    }
}

async fn example() {
    let state = RefCell::new(0);
    [
        &mut one(&state) as &mut dyn Future<Output = _>,
        &mut two(&state),
    ].select().await;
}

fn main() {
    <pasts::ThreadInterrupt as pasts::Interrupt>::block_on(example());
}
