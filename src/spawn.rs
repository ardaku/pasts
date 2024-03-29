use alloc::{sync::Arc, task::Wake, vec::Vec};
use core::{cell::Cell, fmt, future::Future};

use crate::prelude::*;

/// Pasts' executor.
///
/// # Run a Future
/// It's relatively simple to block on a future, and run it to completion:
///
/// ```rust
#[doc = include_str!("../examples/executor.rs")]
/// ```
/// 
/// # Spawn a Future
/// You may spawn tasks on an `Executor`.  Only once all tasks have completed,
/// can [`block_on()`](Executor::block_on()) return.
/// ```rust,no_run
#[doc = include_str!("../examples/spawn.rs")]
/// ```
/// 
/// # Recursive `block_on()`
/// One cool feature about the pasts executor is that you can run it from within
/// the context of another:
/// ```rust
#[doc = include_str!("../examples/recursive.rs")]
/// ```
/// 
/// Or even resume the executor from within it's own context:
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
    /// Spawn a [`LocalBoxNotify`] on this executor.
    ///
    /// Execution of the [`LocalBoxNotify`] will halt after the first poll that
    /// returns [`Ready`].
    #[inline(always)]
    pub fn spawn_notify(&self, n: LocalBoxNotify<'static>) {
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

        // Fuse the future, box it, and push it onto the pool.
        #[cfg(not(feature = "web"))]
        self.spawn_notify(Box::pin(f.fuse()));
    }
}

/// Storage for a task pool.
///
/// # Implementing `Pool` For A Custom Executor
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
    fn push(&self, task: LocalBoxNotify<'static>);

    /// Drain tasks from the thread pool queue.  Should returns true if drained
    /// at least one task.
    fn drain(&self, tasks: &mut Vec<LocalBoxNotify<'static>>) -> bool;
}

/// Trait for implementing the parking / unparking threads.
pub trait Park: Default + Send + Sync + 'static {
    /// The park routine; should put the processor or thread to sleep in order
    /// to save CPU cycles and power, until the hardware tells it to wake up.
    fn park(&self);

    /// Wake the processor or thread.
    fn unpark(&self);
}

#[derive(Default)]
pub struct DefaultPool {
    spawning_queue: Cell<Vec<LocalBoxNotify<'static>>>,
}

impl fmt::Debug for DefaultPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let queue = self.spawning_queue.take();

        f.debug_struct("DefaultPool")
            .field("spawning_queue", &queue)
            .finish()?;
        self.spawning_queue.set(queue);

        Ok(())
    }
}

impl Pool for DefaultPool {
    type Park = DefaultPark;

    // Push onto queue of tasks to spawn.
    #[inline(always)]
    fn push(&self, task: LocalBoxNotify<'static>) {
        let mut queue = self.spawning_queue.take();

        queue.push(task);
        self.spawning_queue.set(queue);
    }

    // Drain from queue of tasks to spawn.
    #[inline(always)]
    fn drain(&self, tasks: &mut Vec<LocalBoxNotify<'static>>) -> bool {
        let mut queue = self.spawning_queue.take();
        let mut drained = queue.drain(..).peekable();
        let has_drained = drained.peek().is_some();

        tasks.extend(drained);
        self.spawning_queue.set(queue);

        has_drained
    }
}

#[cfg(not(feature = "std"))]
#[derive(Copy, Clone, Debug, Default)]
pub struct DefaultPark;

#[cfg(feature = "std")]
#[derive(Debug)]
pub struct DefaultPark(std::sync::atomic::AtomicBool, std::thread::Thread);

#[cfg(feature = "std")]
impl Default for DefaultPark {
    fn default() -> Self {
        Self(
            std::sync::atomic::AtomicBool::new(true),
            std::thread::current(),
        )
    }
}

impl Park for DefaultPark {
    // Park the current thread.
    #[inline(always)]
    fn park(&self) {
        // Only park on std; There is no portable parking for no-std.
        #[cfg(feature = "std")]
        while self.0.swap(true, std::sync::atomic::Ordering::SeqCst) {
            std::thread::park();
        }

        // Hint at spin loop to possibly short sleep on no-std to save CPU time.
        #[cfg(not(feature = "std"))]
        core::hint::spin_loop();
    }

    // Unpark the parked thread
    #[inline(always)]
    fn unpark(&self) {
        // Only unpark on std; Since no-std doesn't park, it's already unparked.
        #[cfg(feature = "std")]
        if self.0.swap(false, std::sync::atomic::Ordering::SeqCst) {
            self.1.unpark();
        }
    }
}

struct Unpark<P: Park>(P);

impl<P: Park> Wake for Unpark<P> {
    #[inline(always)]
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }

    #[inline(always)]
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

#[cfg(not(feature = "web"))]
fn block_on<P: Pool>(f: impl Future<Output = ()> + 'static, pool: &Arc<P>) {
    // Fuse main task
    let f: LocalBoxNotify<'_> = Box::pin(f.fuse());

    // Set up the notify
    let tasks = &mut Vec::new();

    // Set up the park, waker, and context.
    let parky = Arc::new(Unpark(<P as Pool>::Park::default()));
    let waker = parky.clone().into();
    let tasky = &mut Task::from_waker(&waker);

    // Spawn main task
    tasks.push(f);

    // Run the set of futures to completion.
    while !tasks.is_empty() {
        // Poll the set of futures
        let poll = Pin::new(tasks.as_mut_slice()).poll_next(tasky);
        // If no tasks have completed, then park
        let Ready((task_index, ())) = poll else {
            // Initiate execution of any spawned tasks - if no new tasks, park
            if !pool.drain(tasks) {
                parky.0.park();
            }
            continue;
        };

        // Task has completed
        tasks.swap_remove(task_index);
        // Drain any spawned tasks into the pool
        pool.drain(tasks);
    }
}
