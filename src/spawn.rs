// Copyright © 2019-2023 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// - MIT License (https://mit-license.org/)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use alloc::{sync::Arc, task::Wake};
use core::{cell::Cell, fmt, future::Future, marker::PhantomData};

use crate::prelude::*;

/// Implementation for spawning tasks on an executor.
pub trait Spawn: Clone {
    /// Spawn a [`Future`] without the [`Send`] requirement.
    ///
    /// This forces the executor to always run the task on the same thread that
    /// this method is called on.
    fn spawn_local(&self, f: impl Future<Output = ()> + 'static);

    /// Spawn a [`Future`] that is [`Send`].
    ///
    /// This allows the executor to run the task on whatever thread it
    /// determines is most efficient.
    #[inline(always)]
    fn spawn(&self, f: impl Future<Output = ()> + Send + 'static) {
        self.spawn_local(f)
    }
}

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
/// The `Executor` type implements [`Spawn`], which means you can spawn tasks
/// from it.  Only once all tasks have completed, can
/// [`block_on()`](Executor::block_on()) return.
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
pub struct Executor<P: Pool = DefaultPool>(Arc<P>, PhantomData<*mut ()>);

impl Default for Executor {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<P: Pool> Clone for Executor<P> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0), PhantomData)
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
        Self(Arc::new(pool), PhantomData)
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
        block_on(f, self.0);
    }
}

impl<P: Pool> Spawn for Executor<P> {
    #[inline(always)]
    fn spawn_local(&self, f: impl Future<Output = ()> + 'static) {
        // Fuse the future, box it, and push it onto the pool.
        self.0.push(Box::pin(f.fuse()))
    }
}

/// Storage for a task pool.
pub trait Pool {
    /// Type that handles the sleeping / waking of the executor.
    type Park: Park;

    /// Push a task into the thread pool queue.
    fn push(&self, task: LocalBoxNotifier<'static, ()>);

    /// Drain tasks from the thread pool queue.  Should returns true if drained
    /// at least one task.
    fn drain(&self, tasks: &mut Vec<LocalBoxNotifier<'static, ()>>) -> bool;
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
    spawning_queue: Cell<Vec<LocalBoxNotifier<'static, ()>>>,
    park: Arc<DefaultPark>,
}

impl fmt::Debug for DefaultPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let queue = self.spawning_queue.take();

        f.debug_struct("DefaultPool")
            .field("spawning_queue", &queue)
            .field("park", &self.park)
            .finish()?;
        self.spawning_queue.set(queue);

        Ok(())
    }
}

impl Pool for DefaultPool {
    type Park = DefaultPark;

    // Push onto queue of tasks to spawn.
    #[inline(always)]
    fn push(&self, task: LocalBoxNotifier<'static, ()>) {
        let mut queue = self.spawning_queue.take();

        queue.push(task);
        self.spawning_queue.set(queue);
    }

    // Drain from queue of tasks to spawn.
    #[inline(always)]
    fn drain(&self, tasks: &mut Vec<LocalBoxNotifier<'static, ()>>) -> bool {
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
fn block_on<P: Pool>(f: impl Future<Output = ()> + 'static, pool: Arc<P>) {
    // Fuse main task
    let f: LocalBoxNotifier<'_> = Box::pin(f.fuse());

    // Set up the notifier
    let tasks = &mut Vec::new();

    // Set up the park, waker, and context.
    let parky = Arc::new(Unpark(<P as Pool>::Park::default()));
    let waker = parky.clone().into();
    let tasky = &mut Task::from_waker(&waker);

    // Spawn main task
    tasks.push(f);

    // Run the set of futures to completion.
    while !tasks.is_empty() {
        let poll = Pin::new(tasks.as_mut_slice()).poll_next(tasky);
        let Ready((task_index, ())) = poll else {
            if !pool.drain(tasks) {
                parky.0.park();
            }
            continue;
        };
        tasks.swap_remove(task_index);
    }
}
