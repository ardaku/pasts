use std::{
    sync::Arc,
    task::Wake,
    thread::{self, Thread},
};

use pasts::{prelude::*, Executor, Sleep};

/// A waker that wakes up the current thread when called.
struct ThreadExecutor(Thread);

impl Wake for ThreadExecutor {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

impl Sleep for ThreadExecutor {
    fn sleep(&self) {
        thread::park();
    }
}

fn main() {
    // Create an executor.
    let executor = Executor::new(ThreadExecutor(thread::current()));

    // Spawn the future
    executor.spawn(Box::pin(async {
        println!("Hi from inside a future!");
    }));
}
