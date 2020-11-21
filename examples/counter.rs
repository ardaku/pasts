#![forbid(unsafe_code)]

use async_std::task;
use pasts::prelude::*;

use std::{cell::RefCell, time::Duration};

async fn one(state: &RefCell<usize>) {
    println!("Starting task one");
    while *state.borrow() < 5 {
        task::sleep(Duration::new(1, 0)).await;
        let mut state = state.borrow_mut();
        println!("One {}", *state);
        *state += 1;
    }
    println!("Finish task one");
}

async fn two(state: &RefCell<usize>) {
    println!("Starting task two");
    loop {
        task::sleep(Duration::new(2, 0)).await;
        let mut state = state.borrow_mut();
        println!("Two {}", *state);
        *state += 1;
    }
}

async fn example() {
    let state = RefCell::new(0);
    task! {
        let task_one = one(&state);
        let task_two = two(&state);
    }
    [task_one, task_two].select().await;
}

fn main() {
    pasts::spawn(example);
}
