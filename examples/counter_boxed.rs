#![forbid(unsafe_code)]

use async_std::task;
use pasts::prelude::*;

use std::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering::SeqCst},
    time::Duration,
};

async fn one(state: &AtomicUsize) {
    println!("Starting task one");
    while state.load(SeqCst) < 5 {
        task::sleep(Duration::new(1, 0)).await;
        let state_val = state.load(SeqCst);
        println!("One {}", state_val);
        state.store(state_val + 1, SeqCst);
    }
    println!("Finish task one");
}

async fn two(state: &AtomicUsize) {
    println!("Starting task two");
    loop {
        task::sleep(Duration::new(2, 0)).await;
        let state_val = state.load(SeqCst);
        println!("Two {}", state_val);
        state.store(state_val + 1, SeqCst);
    }
}

static STATE: AtomicUsize = AtomicUsize::new(0);

async fn example() {
    let task_one: Pin<Box<dyn Future<Output = ()>>> = Box::pin(one(&STATE));
    let task_two: Pin<Box<dyn Future<Output = ()>>> = Box::pin(two(&STATE));
    [task_one, task_two].select_boxed().await;
}

fn main() {
    pasts::spawn(example);
}
