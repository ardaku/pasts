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
use crate::{Fuse, Join};

// Spawned task queue
#[cfg(all(feature = "std", not(feature = "web")))]
thread_local! {
    // Thread local tasks
    static TASKS: Cell<Vec<Fuse<Local<'static, ()>>>> = Cell::new(Vec::new());

    // Task spawning waker
    static WAKER: Cell<Option<Waker>> = Cell::new(None);
}

/// The implementation of sleeping for an [`Executor`].
///
/// This trait can be used in conjunction with [`Wake`] to create an
/// [`Executor`].
///
/// <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
/// <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
/// <script>hljs.highlightAll();</script>
/// <style> code.hljs { background-color: #000B; } </style>
pub trait Sleep {
    /// The sleep routine; should put the processor or thread to sleep in order
    /// to save CPU cycles and power, until the hardware tells it to wake up.
    fn sleep(&self);
}

/// The implementation of spawning tasks on an [`Executor`].
///
/// <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
/// <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
/// <script>hljs.highlightAll();</script>
/// <style> code.hljs { background-color: #000B; } </style>
pub trait Spawn {
    /// Spawn a [`Future`] on the current thread.
    fn spawn<F: 'static + Future<Output = ()> + Unpin>(self: &Arc<Self>, f: F);
}

impl<T: 'static + Sleep + Wake + Send + Sync> Spawn for T {
    // No std can only spawn one task, so block on it.
    #[cfg(any(not(feature = "std"), feature = "web"))]
    fn spawn<F: 'static + Future<Output = ()> + Unpin>(self: &Arc<T>, fut: F) {
        // Set up the waker and context.
        let waker = self.clone().into();
        let mut cx = TaskCx::from_waker(&waker);

        // Run the future to completion.
        let mut fut = fut;
        while Pin::new(&mut fut).poll(&mut cx).is_pending() {
            self.sleep();
        }
    }

    // Add to the task queue on std
    #[cfg(all(feature = "std", not(feature = "web")))]
    fn spawn<F: 'static + Future<Output = ()> + Unpin>(self: &Arc<T>, fut: F) {
        TASKS.with(|t| {
            let mut tasks = t.take();
            let task: Box<dyn Future<Output = _>> = Box::new(fut);
            tasks.push(Fuse::from(Pin::from(task)));
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
/// 
/// <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
/// <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
/// <script>hljs.highlightAll();</script>
/// <style> code.hljs { background-color: #000B; } </style>
#[derive(Debug)]
pub struct Executor<I: 'static + Spawn + Send + Sync = Exec>(Arc<I>, bool);

impl<I: 'static + Spawn + Send + Sync> Clone for Executor<I> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), true)
    }
}

// Wait for task queue on std when dropping.
#[cfg(all(feature = "std", not(feature = "web")))]
impl<I: 'static + Spawn + Send + Sync> Drop for Executor<I> {
    fn drop(&mut self) {
        // Only run this drop impl if on `StdExecutor` and if original.
        use core::any::Any;
        let exec: Arc<dyn Any + Send + Sync + 'static> = self.0.clone();
        let exec: Arc<Exec> = match exec.downcast() {
            Ok(exec) => exec,
            Err(_) => return,
        };
        if self.1 {
            return;
        }

        struct Tasks(Vec<Fuse<Local<'static, ()>>>, Spawner);

        struct Spawner;

        impl Notifier for Spawner {
            type Event = Fuse<Local<'static, ()>>;

            fn poll_next(&mut self, cx: &mut TaskCx<'_>) -> Poll<Self::Event> {
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

        fn spawn(cx: &mut Tasks, task: Fuse<Local<'static, ()>>) -> Poll<()> {
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
        let mut fut = Join::new(tasks)
            .on(|s| &mut s.1, spawn)
            .on(|s| &mut s.0[..], done);

        // Set up the waker and context.
        let waker = exec.clone().into();
        let mut cx = TaskCx::from_waker(&waker);

        // Run the future to completion.
        while Pin::new(&mut fut).poll(&mut cx).is_pending() {
            exec.sleep();
        }
    }
}

#[cfg(all(feature = "std", not(feature = "web")))]
use self::std::StdExecutor as Exec;

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
use self::web::WebExecutor as Exec;

#[cfg(all(feature = "std", feature = "web"))]
mod web {
    use super::*;

    #[derive(Debug, Copy, Clone)]
    pub struct WebExecutor;

    impl Spawn for WebExecutor {
        fn spawn<F: Future<Output = ()> + 'static>(self: &Arc<Self>, fut: F) {
            wasm_bindgen_futures::spawn_local(fut);
        }
    }

    impl Default for Executor<WebExecutor> {
        fn default() -> Self {
            Self(Arc::new(WebExecutor), false)
        }
    }
}

#[cfg(not(feature = "std"))]
use self::none::InefficientExecutor as Exec;

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

impl<I: 'static + Wake + Sleep + Send + Sync> Executor<I> {
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
    pub fn new(implementation: I) -> Self {
        Self(Arc::new(implementation), false)
    }
}

impl<I: 'static + Spawn + Send + Sync> Executor<I> {
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
    /// # extern crate alloc;
    /// # #[allow(unused_imports)]
    /// # use self::main::*;
    /// # mod main {
    #[doc = include_str!("../examples/spawn/src/main.rs")]
    /// #     pub(super) mod main {
    /// #         pub(in crate) async fn main(executor: pasts::Executor) {
    /// #             super::main(&executor).await
    /// #         }
    /// #     }
    /// # }
    /// # fn main() {
    /// #     let executor = pasts::Executor::default();
    /// #     executor.spawn(Box::pin(self::main::main::main(executor.clone())));
    /// # }
    /// ```
    pub fn spawn(&self, fut: impl Future<Output = ()> + Unpin + 'static) {
        self.0.spawn(fut);
    }
}
