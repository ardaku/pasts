// Pasts
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use alloc::sync::Arc;
use alloc::task::Wake;
use core::future::Future;
use core::task::Context;

// Compensate for Box not being in the prelude on no-std.
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

#[cfg(target_arch = "wasm32")]
use core::{cell::RefCell, pin::Pin};

#[cfg(target_arch = "wasm32")]
type GlobalFuture = RefCell<Pin<Box<dyn Future<Output = ()>>>>;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static FUTURE: GlobalFuture = RefCell::new(Box::pin(async {}));
}

/// The pasts executor.
#[derive(Debug)]
#[allow(missing_copy_implementations)] // For no-std
pub struct Executor {
    // Which thread the executor is runnning on.
    #[cfg(feature = "std")]
    thread: std::thread::Thread,

    // Sleep until interrupt routine.
    sleep: fn(),
    // Wake from interrupt routine.
    wake: fn(),
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    /// Create a new standard executor.
    ///
    /// On no_std, you should use
    /// [`with_custom()`](crate::Executor::with_custom) to get the best
    /// performance.
    pub fn new() -> Self {
        #[cfg(feature = "std")]
        return Self::with_custom(std::thread::park, do_nothing);

        #[cfg(not(feature = "std"))]
        Self::with_custom(do_nothing, do_nothing)
    }

    /// Create an executor with a custom sleep until interrupt and wake from
    /// interrupt routines.
    ///
    /// Useful on embedded devices.
    pub fn with_custom(sleep: fn(), wake: fn()) -> Self {
        Executor {
            #[cfg(feature = "std")]
            thread: std::thread::current(),

            sleep,
            wake,
        }
    }

    /// **std**: Spawn a future on a new thread.
    #[cfg(feature = "std")]
    pub fn spawn<F, T>(self, fut: F) -> std::thread::JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        self.spawn_with(std::thread::Builder::new(), fut).unwrap()
    }

    /// **std**: Spawn a future on a new thread.
    #[cfg(feature = "std")]
    pub fn spawn_with<F, T>(
        self,
        builder: std::thread::Builder,
        fut: F,
    ) -> std::io::Result<std::thread::JoinHandle<T>>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        builder.spawn(move || {
            let mut fut = Box::pin(fut);
            let sleep = self.sleep;
            let waker = Arc::new(self).into();
            let mut cx = Context::from_waker(&waker);

            loop {
                if let std::task::Poll::Ready(output) =
                    fut.as_mut().poll(&mut cx)
                {
                    break output;
                } else {
                    (sleep)();
                }
            }
        })
    }

    /// Run a future to completion on the current thread in an infinite loop.
    pub fn cycle<F: Future<Output = ()> + 'static>(self, fut: F) {
        let mut fut = Box::pin(fut);
        #[cfg(not(target_arch = "wasm32"))]
        let sleep = self.sleep;
        let waker = Arc::new(self).into();
        let cx = &mut Context::from_waker(&waker);

        #[cfg(not(target_arch = "wasm32"))]
        loop {
            // Infinite loop.
            let _ = fut.as_mut().poll(cx);
            (sleep)();
        }

        #[cfg(target_arch = "wasm32")]
        FUTURE.with(move |x| {
            let _ = fut.as_mut().poll(cx);
            *x.borrow_mut() = fut;
        })
    }
}

impl Wake for Executor {
    #[inline(always)]
    fn wake(self: Arc<Self>) {
        Self::wake_by_ref(&self)
    }

    #[inline(always)]
    fn wake_by_ref(self: &Arc<Self>) {
        #[cfg(not(feature = "std"))]
        (self.wake)();

        #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
        self.thread.unpark();

        #[cfg(target_arch = "wasm32")]
        FUTURE.with(move |x| {
            let waker = self.clone().into();
            let mut cx = Context::from_waker(&waker);
            let _ = x.borrow_mut().as_mut().poll(&mut cx);
        })
    }
}

fn do_nothing() {}
