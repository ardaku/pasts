// Copyright © 2019-2022 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use alloc::{sync::Arc, task::Wake};

#[cfg(all(feature = "std", not(feature = "web")))]
use ::std::{cell::Cell, task::Waker};

use crate::prelude::*;
#[cfg(all(feature = "std", not(feature = "web")))]
use crate::{Join, LocalTask, Task};

// Spawned task queue
#[cfg(all(feature = "std", not(feature = "web")))]
thread_local! {
    // Thread local tasks
    static TASKS: Cell<Vec<LocalTask<'static, ()>>> = Cell::new(Vec::new());

    // Task spawning waker
    static WAKER: Cell<Option<Waker>> = Cell::new(None);
}

/// The implementation of sleeping for an [`Executor`].
///
/// This trait can be used in conjunction with [`Wake`] to create an
/// [`Executor`].
///
/// # Example
/// ```rust
#[doc = include_str!("../examples/executor.rs")]
/// ```
pub trait Sleep {
    /// The sleep routine; should put the processor or thread to sleep in order
    /// to save CPU cycles and power, until the hardware tells it to wake up.
    fn sleep(&self);
}

/// The implementation of spawning tasks locally for an [`Executor`].
pub trait SpawnLocal {
    /// Spawn a [`Future`] on the current thread.
    fn spawn_local<F>(self: &Arc<Self>, future: F)
    where
        F: 'static + Future<Output = ()> + Unpin;

    /// Implementation for yielding to the executor.
    ///
    /// The default implementation does nothing.
    fn executor_yield(self: &Arc<Self>) {}
}

impl<T: 'static + Sleep + Wake + Send + Sync> SpawnLocal for T {
    // No std can only spawn one task, so block on it.
    #[cfg(any(not(feature = "std"), feature = "web"))]
    fn spawn_local<F>(self: &Arc<Self>, mut future: F)
    where
        F: 'static + Future<Output = ()> + Unpin,
    {
        // Set up the waker and context.
        let waker = self.clone().into();
        let mut cx = TaskCx::from_waker(&waker);

        // Run the future to completion.
        while Pin::new(&mut future).poll(&mut cx).is_pending() {
            self.sleep();
        }
    }

    // Add to the task queue on std
    #[cfg(all(feature = "std", not(feature = "web")))]
    fn spawn_local<F>(self: &Arc<Self>, future: F)
    where
        F: 'static + Future<Output = ()> + Unpin,
    {
        TASKS.with(|t| {
            let mut tasks = t.take();
            tasks.push(Task::new(future).into());
            t.set(tasks);
        });
        WAKER.with(|w| w.take().map(|w| w.wake()));
    }

    // Go through task queue on std
    #[cfg(all(feature = "std", not(feature = "web")))]
    fn executor_yield(self: &Arc<Self>) {
        struct Tasks(Vec<LocalTask<'static, ()>>, Spawner);

        struct Spawner;

        impl Notifier for Spawner {
            type Event = LocalTask<'static, ()>;

            fn poll_next(
                self: Pin<&mut Self>,
                cx: &mut TaskCx<'_>,
            ) -> Poll<Self::Event> {
                WAKER.with(|w| w.set(Some(cx.waker().clone())));
                TASKS.with(|t| {
                    let mut tasks = t.take();
                    let output = tasks.pop();
                    t.set(tasks);
                    if let Some(task) = output {
                        return Ready(task);
                    }
                    Pending
                })
            }
        }

        fn spawn(tasks: &mut Tasks, task: LocalTask<'static, ()>) -> Poll<()> {
            tasks.0.push(task);
            Pending
        }

        fn done(tasks: &mut Tasks, (id, ()): (usize, ())) -> Poll<()> {
            tasks.0.swap_remove(id);
            if tasks.0.is_empty() { Ready(()) } else { Pending }
        }

        // Set up the future
        let tasks = &mut Tasks(Vec::new(), Spawner);
        let mut fut =
            Join::new(tasks).on(|s| &mut s.1, spawn).on(|s| &mut s.0[..], done);

        // Set up the waker and context.
        let waker = self.clone().into();
        let mut cx = TaskCx::from_waker(&waker);

        // Run the future to completion.
        while Pin::new(&mut fut).poll(&mut cx).is_pending() {
            self.sleep();
        }
    }
}

/// An executor.
///
/// Executors drive [`Future`]s.
#[derive(Debug)]
pub struct Executor<I: 'static + SpawnLocal + Send + Sync>(Arc<I>);

impl<I: 'static + SpawnLocal + Send + Sync> Drop for Executor<I> {
    fn drop(&mut self) {
        self.0.executor_yield();
    }
}

#[cfg(all(feature = "std", not(feature = "web")))]
mod std {
    use ::std::{
        sync::atomic::{AtomicBool, Ordering},
        thread::{self, Thread},
    };

    use super::*;

    #[derive(Debug)]
    pub struct StdExecutor(Thread, AtomicBool);

    impl Sleep for StdExecutor {
        #[inline]
        fn sleep(&self) {
            // Park the current thread.
            while self.1.swap(true, Ordering::SeqCst) {
                std::thread::park();
            }
        }
    }

    impl Wake for StdExecutor {
        #[inline]
        fn wake_by_ref(self: &Arc<Self>) {
            // Unpark the current thread, set the awake? flag if needed.
            if self.1.swap(false, Ordering::SeqCst) {
                self.0.unpark();
            }
        }

        #[inline]
        fn wake(self: Arc<Self>) {
            self.wake_by_ref()
        }
    }

    impl Default for Executor<StdExecutor> {
        fn default() -> Self {
            Self::new(StdExecutor(thread::current(), AtomicBool::new(true)))
        }
    }
}

#[cfg(all(feature = "std", feature = "web"))]
mod web {
    use super::*;

    #[derive(Debug, Copy, Clone)]
    pub struct WebExecutor;

    impl SpawnLocal for WebExecutor {
        fn spawn_local<F>(self: &Arc<Self>, future: F)
        where
            F: Future<Output = ()> + 'static,
        {
            wasm_bindgen_futures::spawn_local(future);
        }
    }

    impl Default for Executor<WebExecutor> {
        fn default() -> Self {
            Self::new(WebExecutor)
        }
    }
}

#[cfg(not(feature = "std"))]
mod none {
    use super::*;

    #[derive(Debug, Copy, Clone)]
    pub struct InefficientExecutor;

    impl Sleep for InefficientExecutor {
        // Never sleep, stay up all night
        #[inline]
        fn sleep(&self) {}
    }

    impl Wake for InefficientExecutor {
        // If you don't sleep, you never wake up
        #[inline]
        fn wake(self: Arc<Self>) {}
    }

    impl Default for Executor<InefficientExecutor> {
        fn default() -> Self {
            Self::new(InefficientExecutor)
        }
    }
}

impl<I: 'static + SpawnLocal + Send + Sync> Executor<I> {
    /// Create a new executor from something implementing both [`SpawnLocal`].
    ///
    /// # Platform-Specific Behavior
    /// If you create an `Executor` in thread-local storage, then the executor
    /// might exit without ever driving the futures spawned on it.  This is
    /// because execution of futures may happen on [`Drop`], which is not
    /// guaranteed for thread local storage.
    ///
    /// **TLDR** If you need to share an executor, wrap it in an [`Arc`],
    /// avoiding thread-local.
    #[inline]
    pub fn new(implementation: I) -> Self {
        Self(Arc::new(implementation))
    }

    /// Spawn an [`Unpin`] future on this executor.
    ///
    /// The program will exit once all spawned futures have completed.
    ///
    /// # Platform-Specific Behavior
    /// On no-std, spawning a future will immediately block on that future,
    /// suspending any currently executing future until the spawned future
    /// finishes.
    ///
    /// # Example
    /// ```rust,no_run
    #[doc = include_str!("../examples/timer.rs")]
    /// ```
    #[inline]
    pub fn spawn(&self, fut: impl Future<Output = ()> + Unpin + 'static) {
        self.0.spawn_local(fut);
    }
}
