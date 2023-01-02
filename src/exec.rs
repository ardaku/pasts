// Copyright © 2019-2023 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// - MIT License (https://mit-license.org/)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use alloc::{sync::Arc, task::Wake};

use self::internal::MainExec;
use crate::prelude::*;

// Spawned task queue
#[cfg(all(not(feature = "no-std"), not(feature = "web")))]
thread_local! {
    // Thread local tasks
    static TASKS: std::cell::Cell<Vec<LocalBoxNotifier<'static, ()>>>
        = std::cell::Cell::new(Vec::new());

    // Task spawning waker
    static WAKER: std::cell::Cell<Option<std::task::Waker>>
        = std::cell::Cell::new(None);
}

/// The implementation of sleeping for an [`Executor`].
///
/// This trait can be used in conjunction with [`Wake`] to create an
/// [`Executor`].
pub trait Sleep: Send + Sync + 'static {
    /// The sleep routine; should put the processor or thread to sleep in order
    /// to save CPU cycles and power, until the hardware tells it to wake up.
    fn sleep(&self);
}

#[cfg(all(not(feature = "no-std"), not(feature = "web")))]
pub trait Spawner: Wake + Sleep {}

#[cfg(all(not(feature = "no-std"), not(feature = "web")))]
impl<T: Wake + Sleep> Spawner for T {}

#[cfg(any(feature = "no-std", feature = "web"))]
pub trait Spawner {}

#[cfg(any(feature = "no-std", feature = "web"))]
impl<T> Spawner for T {}

/// The implementation of spawning tasks on an [`Executor`].
pub trait Spawn: Spawner {
    /// Spawn a [`Future`] on the current thread.
    fn spawn<F: 'static + Future<Output = ()>>(self: &Arc<Self>, f: F);
}

impl<T: Sleep + Wake> Spawn for T {
    // No std can only spawn one task, so block on it.
    #[cfg(any(feature = "no-std", feature = "web"))]
    fn spawn<F: 'static + Future<Output = ()>>(self: &Arc<T>, fut: F) {
        // Set up the waker and context.
        let waker = self.clone().into();
        let mut exec = Exec::from_waker(&waker);

        #[cfg(feature = "no-std")]
        pin_utils::pin_mut!(fut);

        #[cfg(feature = "web")]
        let mut fut = Box::pin(fut);

        // Run the future to completion.
        while Pin::new(&mut fut).poll(&mut exec).is_pending() {
            self.sleep();
        }
    }

    // Add to the task queue on std
    #[cfg(all(not(feature = "no-std"), not(feature = "web")))]
    fn spawn<F: 'static + Future<Output = ()>>(self: &Arc<T>, fut: F) {
        TASKS.with(|t| {
            let mut tasks = t.take();
            tasks.push(Box::pin(fut.fuse()));
            t.set(tasks);
        });
        WAKER.with(|w| w.take().map(|w| w.wake()));
    }
}

/// An executor.
///
/// Executors drive [`Future`]s.
///
/// # Example
/// ```rust,no_run
#[doc = include_str!("../examples/timer.rs")]
/// ```
#[derive(Debug)]
pub struct Executor<I: Spawn = MainExec>(Arc<I>, bool);

impl Default for Executor {
    fn default() -> Self {
        Self(MainExec::default().into(), false)
    }
}

impl<I: Spawn> Clone for Executor<I> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), true)
    }
}

// Wait for task queue on std when dropping.
#[cfg(all(not(feature = "no-std"), not(feature = "web")))]
impl<I: Spawn> Drop for Executor<I> {
    fn drop(&mut self) {
        // Only run this drop impl if on std feature and if original.
        if self.1 {
            return;
        }

        struct Tasks(Vec<LocalBoxNotifier<'static, ()>>, Spawner);

        struct Spawner;

        impl Notifier for Spawner {
            type Event = LocalBoxNotifier<'static, ()>;

            fn poll_next(
                self: Pin<&mut Self>,
                e: &mut Exec<'_>,
            ) -> Poll<Self::Event> {
                WAKER.with(|w| w.set(Some(e.waker().clone())));
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

        fn spawn(
            cx: &mut Tasks,
            task: LocalBoxNotifier<'static, ()>,
        ) -> Poll<()> {
            cx.0.push(task);
            Pending
        }

        fn done(cx: &mut Tasks, (id, ()): (usize, ())) -> Poll<()> {
            cx.0.swap_remove(id);
            if cx.0.is_empty() {
                Ready(())
            } else {
                Pending
            }
        }

        // Set up the future
        let tasks = &mut Tasks(Vec::new(), Spawner);
        let mut fut = crate::Join::new(tasks)
            .on(|s| &mut s.1, spawn)
            .on(|s| &mut s.0[..], done);

        // Set up the waker and context.
        let waker = self.0.clone().into();
        let mut e = Exec::from_waker(&waker);

        // Run the future to completion.
        while Pin::new(&mut fut).poll(&mut e).is_pending() {
            self.0.sleep();
        }
    }
}

#[cfg(all(not(feature = "no-std"), not(feature = "web")))]
mod internal {
    use ::std::{
        sync::atomic::{AtomicBool, Ordering},
        thread::{self, Thread},
    };

    use super::*;

    #[derive(Debug)]
    pub struct MainExec(Thread, AtomicBool);

    impl Sleep for MainExec {
        #[inline]
        fn sleep(&self) {
            // Park the current thread.
            while self.1.swap(true, Ordering::SeqCst) {
                std::thread::park();
            }
        }
    }

    impl Wake for MainExec {
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

    impl Default for MainExec {
        fn default() -> Self {
            Self(thread::current(), AtomicBool::new(true))
        }
    }
}

#[cfg(all(not(feature = "no-std"), feature = "web"))]
mod internal {
    use super::*;

    #[derive(Debug, Copy, Clone, Default)]
    pub struct MainExec;

    impl Spawn for MainExec {
        fn spawn<F: Future<Output = ()> + 'static>(self: &Arc<Self>, fut: F) {
            wasm_bindgen_futures::spawn_local(fut);
        }
    }
}

#[cfg(feature = "no-std")]
mod internal {
    use super::*;

    #[derive(Debug, Copy, Clone, Default)]
    pub struct MainExec;

    impl Sleep for MainExec {
        // Never sleep, stay up all night
        fn sleep(&self) {
            core::hint::spin_loop();
        }
    }

    impl Wake for MainExec {
        // If you don't sleep, you never wake up
        fn wake(self: Arc<Self>) {}
    }
}

impl<I: Wake + Sleep> Executor<I> {
    /// Create a new executor from something implementing both [`Wake`] and
    /// [`Sleep`].
    ///
    /// # Platform-Specific Behavior
    /// Execution of futures happens on [`Drop`] of the original (not cloned)
    /// `Executor` when *`std`* is enabled, and *`web`* is not.
    ///
    /// # Example
    /// ```rust
    #[doc = include_str!("../examples/executor.rs")]
    /// ```
    pub fn new(implementation: impl Into<Arc<I>>) -> Self {
        Self(implementation.into(), false)
    }
}

impl<I: Spawn> Executor<I> {
    /// Spawn a future on this executor.
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
    #[doc = include_str!("../examples/spawn.rs")]
    /// ```
    pub fn spawn(&self, fut: impl Future<Output = ()> + 'static) {
        self.0.spawn(fut);
    }
}
