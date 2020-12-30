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
    process,
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
    fn execute<T, F: Future<Output = T>>(&mut self, f: F) -> T {
        let mut f = Box::pin(f);
        // Get a waker and context for this executor.
        crate::util::waker(self, |cx| {
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
