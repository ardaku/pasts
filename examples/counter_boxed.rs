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
    let mut value = state.load(SeqCst);
    task::sleep(Duration::from_millis(10)).await;
    while value < 5 {
        task::sleep(Duration::new(1, 0)).await;
        value = state.fetch_add(1, SeqCst) + 1;
        println!("One {}", value - 1);
    }
    println!("Finish task one");
}

async fn two(state: &AtomicUsize) {
    println!("Starting task two");
    let mut value;
    loop {
        task::sleep(Duration::new(2, 0)).await;
        value = state.fetch_add(1, SeqCst);
        println!("Two {}", value);
    }
}

static STATE: AtomicUsize = AtomicUsize::new(0);

async fn example() {
    let mut task_one: Pin<Box<dyn Future<Output = ()>>> = Box::pin(one(&STATE));
    let mut task_two: Pin<Box<dyn Future<Output = ()>>> = Box::pin(two(&STATE));
    poll![task_one, task_two].await;
}

fn main() {
    exec!(example());
}
