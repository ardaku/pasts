// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::future::Future;

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Condvar, Mutex,
    },
    task::Poll,
};

#[cfg(any(target_arch = "wasm32", not(feature = "std")))]
use alloc::boxed::Box;
#[cfg(any(target_arch = "wasm32", not(feature = "std")))]
use core::{cell::RefCell, pin::Pin};

#[cfg(any(target_arch = "wasm32", not(feature = "std")))]
pub(crate) struct Exec(RefCell<Option<Pin<Box<dyn Future<Output = ()>>>>>);

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
pub(crate) struct Exec {
    // The thread-safe waking mechanism: part 1
    mutex: Mutex<()>,
    // The thread-safe waking mechanism: part 2
    cvar: Condvar,
    // Flag set to verify `Condvar` actually woke the executor.
    state: AtomicBool,
}

impl Exec {
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    pub(crate) fn new() -> Self {
        Self {
            mutex: Mutex::new(()),
            cvar: Condvar::new(),
            state: AtomicBool::new(true),
        }
    }

    #[cfg(any(target_arch = "wasm32", not(feature = "std")))]
    pub(crate) fn new() -> Self {
        Self(RefCell::new(None))
    }

    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    pub(crate) fn wake(&self) {
        // Wake the task running on a separate thread via CondVar
        if !self.state.compare_and_swap(false, true, Ordering::SeqCst) {
            // We notify the condvar that the value has changed.
            self.cvar.notify_one();
        }
    }

    #[cfg(any(target_arch = "wasm32", not(feature = "std")))]
    pub(crate) fn wake(&self) {
        // Wake the task running on this thread - one pass through executor.
        crate::util::waker(self, |cx| {
            self.0
                .borrow_mut()
                .as_mut()
                .unwrap()
                .as_mut()
                .poll(cx)
                .is_pending()
        });
    }

    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    #[allow(unsafe_code)] // Needed to use `task!()` macro within this crate.
    fn execute<T, F: Future<Output = T>>(&mut self, f: F) -> T {
        // Get a waker and context for this executor.
        crate::util::waker(self, |cx| {
            crate::task!(let f = f);
            loop {
                // Exit with future output, on future completion, otherwise…
                if let Poll::Ready(value) = f.as_mut().poll(cx) {
                    break value;
                }
                // Put the thread to sleep until wake() is called.
                let mut guard = self.mutex.lock().unwrap();
                while !self.state.compare_and_swap(
                    true,
                    false,
                    Ordering::SeqCst,
                ) {
                    guard = self.cvar.wait(guard).unwrap();
                }
            }
        })
    }

    #[cfg(any(target_arch = "wasm32", not(feature = "std")))]
    fn execute<F: Future<Output = ()> + 'static>(&self, f: F) {
        // Set the future for this executor.
        *self.0.borrow_mut() = Some(Box::pin(f));
        // Begin Executor
        self.wake();
    }
}

/// Execute futures concurrently; in parallel if the target supports it,
/// otherwise asynchronously.
///
/// Similar to [`poll!()`](crate::poll!), except that you can call it from
/// synchronous code, and after calling it once, future calls will panic.
/// The program will *exit* after the first future returns
/// [`Poll::Ready`](std::task::Poll::Ready).  That means all threads and
/// single-threaded executorr started by this macro will run for the remainder
/// of the program, giving them a `'static` lifetime.
///
/// # Example
/// ```rust
/// async fn async_main() {
///     /* your code here */
/// }
///
/// // Note that you may add multiple concurrent async_main()s.
/// pasts::exec!(async_main());
/// ```
#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
#[macro_export]
macro_rules! exec {
    ($e:expr $(,)?) => {
        $crate::_block_on($e);
    };
    ($e:expr, $($f:expr),* $(,)?) => {
        $( std::thread::spawn(|| $crate::_block_on($f)); )*
        $crate::_block_on($e);
    };
}

/// Execute futures concurrently; in parallel if the target supports it,
/// otherwise asynchronously.
///
/// Similar to [`poll!()`](crate::poll!), except that you can call it from
/// synchronous code, and after calling it once, future calls will panic.
/// The program will *exit* after the first future returns
/// [`Poll::Ready`](std::task::Poll::Ready).  That means all threads and
/// single-threaded executorr started by this macro will run for the remainder
/// of the program, giving them a `'static` lifetime.
///
/// # Example
/// ```rust
/// async fn async_main() {
///     /* your code here */
/// }
///
/// // Note that you may add multiple concurrent async_main()s.
/// pasts::exec!(async_main());
/// ```
#[cfg(any(target_arch = "wasm32", not(feature = "std")))]
#[macro_export]
macro_rules! exec {
    ($e:expr $(,)?) => {
        $crate::_block_on($e);
    };
    ($e:expr, $($f:expr),* $(,)?) => {
        $crate::_block_on($e);
        $( $crate::_block_on($f); )*
    };
}

/// Execute a future by spawning an asynchronous task.
#[doc(hidden)]
pub fn _block_on<F: Future<Output = ()> + 'static>(f: F) {
    // Can start tasks on their own threads.
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    {
        let mut exec = Exec::new();
        exec.execute(f);
        std::process::exit(0);
    }

    // Can allocate task queue.
    #[cfg(any(target_arch = "wasm32", not(feature = "std")))]
    {
        crate::util::exec().execute(f);
    }
}