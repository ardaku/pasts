// Pasts
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

// This is how you use `Condvar`s, it's in the std library docs
#![allow(clippy::mutex_atomic)]

use core::future::Future;

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
use std::{
    process,
    sync::{Condvar, Mutex},
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
    mutex: Mutex<bool>,
    // The thread-safe waking mechanism: part 2
    cvar: Condvar,
}

impl Exec {
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    pub(crate) fn new() -> Self {
        Self {
            mutex: Mutex::new(true),
            cvar: Condvar::new(),
        }
    }

    #[cfg(any(target_arch = "wasm32", not(feature = "std")))]
    pub(crate) fn new() -> Self {
        Self(RefCell::new(None))
    }

    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    pub(crate) fn wake(&self) {
        // Keep mutex locked for the remainder of this function call.
        let mut sleeping = self.mutex.lock().unwrap();
        // Wake the task running on a separate thread via CondVar
        *sleeping = false;
        // We notify the condvar that the value has changed.
        self.cvar.notify_one();
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
    fn execute<F: Future<Output = ()>>(&mut self, f: F) {
        let mut f = Box::pin(f);
        // Get a waker and context for this executor.
        crate::util::waker(self, |cx| {
            loop {
                // Exit with future output, on future completion, otherwise…
                if let Poll::Ready(value) = f.as_mut().poll(cx) {
                    break value;
                }
                // Put the thread to sleep until wake() is called.
                let sleeping = self.mutex.lock().unwrap();
                *self.cvar.wait_while(sleeping, |p| *p).unwrap() = true;
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

/// Execute a future on the current thread.
///
/// Upon completion of the future, the program will exit.  This allows for some
/// optimizations and simplification of code (as well as behavioral consistency
/// on Web Assembly.  You may call `block_on()` on multiple threads to build an
/// asynchronous thread pool.
pub fn block_on<F: Future<Output = ()> + 'static>(f: F) {
    // Can start tasks on their own threads.
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    {
        let mut exec = Exec::new();
        exec.execute(f);
        process::exit(0);
    }

    // Can allocate task queue.
    #[cfg(any(target_arch = "wasm32", not(feature = "std")))]
    {
        crate::util::exec().execute(f);
    }
}

/// Macro to remove boilerplate for executing an asynchronous event loop.
///
/// Argument is an async expression that runs continuously in a loop.
#[macro_export]
macro_rules! exec {
    ($exec:expr) => {{
        $crate::block_on(async move {
            loop {
                $exec
            }
        });
    }};
}
