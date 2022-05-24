// Copyright © 2019-2022 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use alloc::{boxed::Box, sync::Arc, task::Wake};

use crate::prelude::*;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static FUT: (
        std::cell::RefCell<Pin<Box<dyn Future<Output = ()>>>>,
        std::task::Waker,
    ) = (
        std::cell::RefCell::new(Box::pin(std::future::pending())),
        Arc::new(Woke(Exec())).into(),
    );
}

// Internal waker type.
struct Woke<E: Executor>(E);

// Always call executor's wake() method.
impl<E: Executor> Wake for Woke<E> {
    #[inline(always)]
    fn wake(self: Arc<Self>) {
        self.0.wake()
    }

    #[inline(always)]
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.wake()
    }
}

/// Trait for implementing custom executors.  Useful when targetting no-std.
pub trait Executor: Send + Sync + 'static {
    /// The sleep routine; should put the processor or thread to sleep in order
    /// to save CPU cycles and power, until the hardware tells it to wake up.
    fn sleep(&self);

    /// The wake routine; should wake the processor or thread.  If the hardware
    /// is already waked up automatically, this doesn't need to be implemented.
    #[inline(always)]
    fn wake(&self) {}
}

impl<T> BlockOn for T where T: Sized + Executor {}

/// Trait that implements `block_on()` and `block_on_pinned()` methods for an
/// [`Executor`]
pub trait BlockOn: Sized + Executor {
    /// Block on an unpin future on the current thread.
    #[inline(always)]
    fn block_on_pinned<F>(self, future: F)
    where
        F: Future<Output = ()> + Unpin + 'static,
    {
        #[cfg(target_arch = "wasm32")]
        self.block_on(future);

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Pin the future.
            let mut f = future;
            // Put the executor on the heap.
            let executor = Arc::new(Woke(self));
            // Convert executor into a waker.
            let waker = executor.clone().into();
            // Create a context from the waker.
            let cx = &mut Context::from_waker(&waker);

            // If blocking is allowed, loop while blocking.
            loop {
                // First, poll
                if let Ready(it) = Pin::new(&mut f).poll(cx) {
                    break it;
                }
                // Next, wait for wake up completes before polling again.
                executor.0.sleep();
            }
        }
    }

    /// Block on a future on the current thread (puts the future on the heap).
    #[inline(always)]
    fn block_on<F: Future<Output = ()> + 'static>(self, future: F) {
        // WebAssembly can't block, so poll once, then return.
        #[cfg(target_arch = "wasm32")]
        let _ = FUT.with(move |(f, e)| {
            *f.borrow_mut() = Box::pin(future);
            f.borrow_mut().as_mut().poll(&mut Context::from_waker(&e))
        });

        // Pin and block on the future.
        #[cfg(not(target_arch = "wasm32"))]
        self.block_on_pinned(Box::pin(future));
    }
}

// Default executor implementation.
#[derive(Debug)]
struct Exec(
    // On std, store which thread, as well as an "awake?" flag.
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    (std::thread::Thread, std::sync::atomic::AtomicBool),
);

impl Executor for Exec {
    #[inline(always)]
    fn sleep(&self) {
        // On std, park the current thread, without std do nothing.
        #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
        while self.0 .1.swap(true, std::sync::atomic::Ordering::SeqCst) {
            std::thread::park();
        }
    }

    #[inline(always)]
    fn wake(&self) {
        #[cfg(target_arch = "wasm32")]
        let _ = FUT.with(move |(f, e)| {
            f.borrow_mut().as_mut().poll(&mut Context::from_waker(&e))
        });

        // On std, unpark the current thread, setting the awake? flag if needed.
        #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
        if self.0 .1.swap(false, std::sync::atomic::Ordering::SeqCst) {
            self.0 .0.unpark();
        }
    }
}

/// Run a future to completion on the current thread.
///
/// # Platform-Specific Behavior
/// On WebAssembly, this function returns immediately instead of blocking
/// because you're not supposed to block in a web browser.
///
/// # Example
/// ```rust,no_run
#[doc = include_str!("../examples/timer.rs")]
/// ```
pub fn block_on<F: Future<Output = ()> + 'static>(future: F) {
    // On std, associate the current thread.
    Exec(
        #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
        (
            std::thread::current(),
            std::sync::atomic::AtomicBool::new(true),
        ),
    )
    .block_on(future)
}
