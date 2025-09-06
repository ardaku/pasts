use alloc::{sync::Arc, task::Wake, vec::Vec};
use core::fmt;

use crate::{
    park::Park,
    pool::{DefaultPool, Pool},
    prelude::*,
};

/// Pasts' executor.
///
/// # Run a Future
///
/// It's relatively simple to block on a future, and run it to completion:
///
/// ```rust
#[doc = include_str!("../examples/executor.rs")]
/// ```
/// 
/// # Spawn a Future
///
/// You may spawn tasks on an `Executor`.  Only once all tasks have completed,
/// can [`block_on()`](Executor::block_on()) return.
///
/// ```rust,no_run
#[doc = include_str!("../examples/spawn.rs")]
/// ```
/// 
/// # Recursive `block_on()`
///
/// One cool feature about the pasts executor is that you can run it from within
/// the context of another:
///
/// ```rust
#[doc = include_str!("../examples/recursive.rs")]
/// ```
/// 
/// Or even resume the executor from within it's own context:
///
/// ```rust
#[doc = include_str!("../examples/resume.rs")]
/// ```
pub struct Executor<P: Pool = DefaultPool>(Arc<P>);

impl Default for Executor {
    fn default() -> Self {
        Self::new(DefaultPool::default())
    }
}

impl<P: Pool> Clone for Executor<P> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<P: Pool + fmt::Debug> fmt::Debug for Executor<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Executor").field(&self.0).finish()
    }
}

impl<P: Pool> Executor<P> {
    /// Create a new executor that can only spawn tasks from the current thread.
    ///
    /// Custom executors can be built by implementing [`Pool`].
    #[inline(always)]
    pub fn new(pool: P) -> Self {
        Self(Arc::new(pool))
    }

    /// Block on a future and return it's result.
    ///
    /// # Platform-Specific Behavior
    ///
    /// When building with feature _`web`_, spawns task and returns
    /// immediately instead of blocking.
    #[inline(always)]
    pub fn block_on(self, f: impl Future<Output = ()> + 'static) {
        #[cfg(feature = "web")]
        wasm_bindgen_futures::spawn_local(f);

        #[cfg(not(feature = "web"))]
        block_on(f, &self.0);
    }
}

impl<P: Pool> Executor<P> {
    /// Spawn a [`LocalBoxFuture`] on this executor.
    ///
    /// Execution of the [`LocalBoxFuture`] will halt after the first poll that
    /// returns [`Ready`].
    #[inline(always)]
    pub fn spawn_future(&self, n: LocalBoxFuture<'static>) {
        // Convert the notify into a future and spawn on wasm_bindgen_futures
        #[cfg(feature = "web")]
        wasm_bindgen_futures::spawn_local(async move {
            let mut n = n;

            n.next().await;
        });

        // Push the notify onto the pool.
        #[cfg(not(feature = "web"))]
        self.0.push(n);
    }

    /// Box and spawn a future on this executor.
    #[inline(always)]
    pub fn spawn_boxed(&self, f: impl Future<Output = ()> + 'static) {
        // Spawn the future on wasm_bindgen_futures
        #[cfg(feature = "web")]
        wasm_bindgen_futures::spawn_local(f);

        // Box the future, and push it onto the pool.
        #[cfg(not(feature = "web"))]
        self.spawn_future(Box::pin(f));
    }
}

struct Unpark<P: Park>(P);

impl<P: Park> Wake for Unpark<P> {
    #[inline(always)]
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    #[inline(always)]
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

#[cfg(not(feature = "web"))]
fn block_on<P: Pool>(f: impl Future<Output = ()> + 'static, pool: &Arc<P>) {
    // Box and pin main task
    let f: LocalBoxFuture<'_> = Box::pin(f);
    // Set up the notify
    let tasks = &mut Vec::new();
    // Set up the park, waker, and context
    let parky = Arc::new(Unpark(<P as Pool>::Park::default()));
    let waker = parky.clone().into();
    let tasky = &mut Task::from_waker(&waker);
    // Which task's turn it is (for basic fairness)
    let mut index = 0;

    // Spawn main task
    tasks.push(f);

    // Run the set of futures to completion.
    while !tasks.is_empty() {
        // Wrap index
        index %= tasks.len();

        // Poll the entire set of futures on wake
        let poll = 'poll: {
            for (i, this) in tasks.iter_mut().skip(index).enumerate() {
                if let Ready(()) = Pin::new(this).poll(tasky) {
                    break 'poll Ready(i);
                }
            }
            
            for (i, this) in tasks.iter_mut().take(index).enumerate() {
                if let Ready(()) = Pin::new(this).poll(tasky) {
                    break 'poll Ready(i);
                }
            }

            // Take turns which task polls first
            index += 1;
            break 'poll Pending;
        };
        // If no tasks have completed, then park
        let Ready(task_index) = poll else {
            // Initiate execution of any spawned tasks - if no new tasks, park
            if !pool.drain(tasks) {
                parky.0.park();
            }

            continue;
        };

        // Task has completed, drop it
        drop(tasks.swap_remove(task_index));
        // Drain any spawned tasks into the pool
        pool.drain(tasks);
    }
}
