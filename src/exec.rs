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
use core::sync::atomic::AtomicBool;
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
    static FUT: (GlobalFuture, std::task::Waker) = (
        RefCell::new(Box::pin(async {})), Arc::new(Waker()).into()
    );
}

/// The pasts executor.
#[derive(Debug)]
#[allow(missing_copy_implementations)] // For no-std
pub struct Executor {
    // Store waker inside executor if not targetting WebAssembly.
    #[cfg(not(target_arch = "wasm32"))]
    waker: Arc<Waker>,
    // Sleep until interrupt routine.
    #[cfg(not(target_arch = "wasm32"))]
    sleep: fn(),
}

impl Default for Executor {
    #[inline(always)]
    fn default() -> Self {
        Executor {
            #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
            waker: Arc::new(Waker(std::thread::current())),
            #[cfg(not(feature = "std"))]
            waker: Arc::new(Waker(do_nothing)),

            #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
            sleep: std::thread::park,
            #[cfg(not(feature = "std"))]
            sleep: do_nothing,
        }
    }
}

impl Executor {
    /// Create executor with custom sleep until interrupt and wake from
    /// interrupt routines.  You should always call this method in no-std
    /// environments.  This method returns `Executor::default()` when the *std*
    /// feature is enabled.
    #[inline(always)]
    pub fn with(sleep: fn(), wake: fn()) -> Self {
        // Hide unused warnings when *std* feature enabled.
        let (_sleep, _wake) = (sleep, wake);
        #[cfg(not(feature = "std"))]
        return Self {
            waker: Arc::new(Waker(_wake)),
            sleep: _sleep,
        };
        #[cfg(feature = "std")]
        Self::default()
    }

    /// Run a future to completion on the current thread.
    ///
    /// # Platform-Specific Behavior
    /// In WebAssembly, this function returns immediately instead of blocking,
    /// since blocking is not supported.  Call this function as the last thing
    /// in your `main()`, and it will continue to execute the task (yielding to
    /// the JavaScript executor once `main()` completes).
    #[inline(always)]
    pub fn block_on<F: Future<Output = ()> + 'static>(self, fut: F) {
        let mut fut = Box::pin(fut);

        #[cfg(not(target_arch = "wasm32"))]
        {
            let waker = self.waker.into();
            let context = &mut Context::from_waker(&waker);
            while fut.as_mut().poll(context).is_pending() {
                (self.sleep)();
            }
        }

        #[cfg(target_arch = "wasm32")]
        FUT.with(move |(x, w)| {
            let _ = fut.as_mut().poll(&mut Context::from_waker(&w));
            *x.borrow_mut() = fut;
        });
    }
}

// The internal waker representation that goes with the executor.
#[derive(Debug)]
struct Waker(
    #[cfg(not(feature = "std"))] fn(),
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    std::thread::Thread,
);

impl Wake for Waker {
    #[inline(always)]
    fn wake(self: Arc<Self>) {
        Self::wake_by_ref(&self)
    }

    #[inline(always)]
    fn wake_by_ref(self: &Arc<Self>) {
        #[cfg(not(feature = "std"))]
        (self.0)();

        #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
        self.0.unpark();

        #[cfg(target_arch = "wasm32")]
        let _ = FUT.with(|(x, w)| {
            x.borrow_mut().as_mut().poll(&mut Context::from_waker(&w))
        });
    }
}

#[cfg(not(feature = "std"))]
fn do_nothing() {}
