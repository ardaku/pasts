use std::{
    cell::Cell,
    thread::{self, Thread},
};

use pasts::{prelude::*, Executor, Park, Pool};

#[derive(Default)]
struct SingleThreadedPool {
    spawning_queue: Cell<Vec<LocalBoxNotifier<'static>>>,
}

impl Pool for SingleThreadedPool {
    type Park = ThreadPark;

    fn push(&self, task: LocalBoxNotifier<'static>) {
        let mut queue = self.spawning_queue.take();

        queue.push(task);
        self.spawning_queue.set(queue);
    }

    fn drain(&self, tasks: &mut Vec<LocalBoxNotifier<'static>>) -> bool {
        let mut queue = self.spawning_queue.take();
        let mut drained = queue.drain(..).peekable();
        let has_drained = drained.peek().is_some();

        tasks.extend(drained);
        self.spawning_queue.set(queue);

        has_drained
    }
}

struct ThreadPark(Thread);

impl Default for ThreadPark {
    fn default() -> Self {
        Self(thread::current())
    }
}

impl Park for ThreadPark {
    fn park(&self) {
        std::thread::park();
    }

    fn unpark(&self) {
        self.0.unpark();
    }
}

fn main() {
    // Create a custom executor.
    let executor = Executor::new(SingleThreadedPool::default());

    // Block on a future
    executor.block_on(async {
        println!("Hi from inside a future!");
    });
}
