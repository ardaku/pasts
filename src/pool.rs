use alloc::vec::Vec;
use core::{cell::Cell, fmt};

use crate::{LocalBoxFuture, Park, park::DefaultPark};

/// Storage for a task pool.
///
/// # Implementing `Pool` For A Custom Executor
///
/// This example shows how to create a custom single-threaded executor using
/// [`Executor::new()`].
///
/// ```rust
#[doc = include_str!("../examples/pool.rs")]
/// ```
pub trait Pool {
    /// Type that handles the sleeping / waking of the executor.
    type Park: Park;

    /// Push a task into the thread pool queue.
    fn push(&self, task: LocalBoxFuture<'static>);

    /// Drain tasks from the thread pool queue.  Should returns true if drained
    /// at least one task.
    fn drain(&self, tasks: &mut Vec<LocalBoxFuture<'static>>) -> bool;
}

#[derive(Default)]
pub struct DefaultPool {
    spawning_queue: Cell<Vec<LocalBoxFuture<'static>>>,
}

impl fmt::Debug for DefaultPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let queue = self.spawning_queue.take();

        f.debug_struct("DefaultPool")
            .field("spawning_queue.len()", &queue.len())
            .finish()?;
        self.spawning_queue.set(queue);
        Ok(())
    }
}

impl Pool for DefaultPool {
    type Park = DefaultPark;

    // Push onto queue of tasks to spawn.
    #[inline(always)]
    fn push(&self, task: LocalBoxFuture<'static>) {
        let mut queue = self.spawning_queue.take();

        queue.push(task);
        self.spawning_queue.set(queue);
    }

    // Drain from queue of tasks to spawn.
    #[inline(always)]
    fn drain(&self, tasks: &mut Vec<LocalBoxFuture<'static>>) -> bool {
        let mut queue = self.spawning_queue.take();
        let mut drained = queue.drain(..).peekable();
        let has_drained = drained.peek().is_some();

        tasks.extend(drained);
        self.spawning_queue.set(queue);
        has_drained
    }
}
