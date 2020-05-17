// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(clippy::mutex_atomic)]

use crate::Executor;

use std::{
    cell::Cell,
    fmt::{Debug, Error, Formatter},
    mem::MaybeUninit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Condvar, Mutex, Once,
    },
};

#[derive(Debug)]
struct CvarExecInternal {
    /// The thread-safe waking mechanism
    mutex: Mutex<()>,
    ///  The thread-safe waking mechanism
    cvar: Condvar,
}

impl Debug for CvarExec {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "CvarExec")
    }
}

/// **std** feature required.  A thread-safe executor that uses a `Condvar` to
/// put the thread to sleep when the future is pending.
///
/// If you can use std, use this `Executor`.
pub struct CvarExec {
    // Runtime intialization check
    once: Once,
    // Runtime initialized data.
    internal: Cell<MaybeUninit<CvarExecInternal>>,
    /// Flag set to verify `Condvar` actually woke the executor.
    state: AtomicBool,
}

#[allow(unsafe_code)]
unsafe impl Sync for CvarExec {}

impl CvarExec {
    /// Construct a new thread-safe executor (one that can be awoken from other
    /// threads and won't crash on wake after thread is done).
    #[inline]
    pub const fn new() -> Self {
        CvarExec {
            once: Once::new(),
            internal: Cell::new(MaybeUninit::uninit()),
            state: AtomicBool::new(true),
        }
    }
}

impl Drop for CvarExec {
    #[inline]
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        if self.is_used() {
            let internal = Cell::new(MaybeUninit::uninit());
            self.internal.swap(&internal);
            let _ = unsafe { internal.into_inner().assume_init() };
        }
    }
}

#[allow(unsafe_code)]
impl Executor for CvarExec {
    #[inline]
    unsafe fn trigger_event(&self) {
        // Set wake flag.
        if self.state.compare_and_swap(false, true, Ordering::SeqCst) == false {
            // We notify the condvar that the value has changed.
            (*(*self.internal.as_ptr()).as_ptr()).cvar.notify_one();
        }
    }

    #[inline]
    unsafe fn wait_for_event(&self) {
        self.once.call_once(|| {
            self.internal.set(MaybeUninit::new(CvarExecInternal {
                mutex: Mutex::new(()),
                cvar: Condvar::new(),
            }));
        });

        let internal = (*self.internal.as_ptr()).as_ptr();

        // Wait for event(s) to get triggered.
        let mut guard = (*internal).mutex.lock().unwrap();
        while self.state.compare_and_swap(true, false, Ordering::SeqCst)
            == false
        {
            guard = (*internal).cvar.wait(guard).unwrap();
        }
    }

    #[inline]
    fn is_used(&self) -> bool {
        self.once.is_completed()
    }
}
