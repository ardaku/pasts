// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    task::{RawWaker, RawWakerVTable, Waker},
    cell::RefCell,
};

#[cfg(not(target_arch = "wasm32"))]
use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
use std::sync::{Condvar, Mutex};

/// An executor for `Future`s.
#[allow(unsafe_code)]
pub trait Executor: 'static + Send + Sync + Sized {
    /// Cause `wait_for_event()` to return.
    ///
    /// # Safety
    /// This method is marked `unsafe` because it must only be called from a
    /// `Waker`.  This is guaranteed by the `block_on()` method.
    unsafe fn trigger_event(&self);
    /// Blocking wait until an event is triggered with `trigger_event`.  This
    /// function should put the current thread or processor to sleep to save
    /// power consumption.
    ///
    /// # Safety
    /// This function should only be called by one executor.  On the first call
    /// to this method, all following calls to `is_used()` should return `true`.
    /// This method is marked `unsafe` because only one thread and one executor
    /// can call it (ever!).  This is guaranteed by the `block_on()` method.
    unsafe fn wait_for_event(&self);
    /// Should return true if `wait_for_event` has been called, false otherwise.
    fn is_used(&self) -> bool;
}

// Executor data.
struct Exec {
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    // The thread-safe waking mechanism: part 1
    mutex: Mutex<()>,

    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    // The thread-safe waking mechanism: part 2
    cvar: Condvar,

    #[cfg(target_arch = "wasm32")]
    // Pinned future.
    future: Option<Pin<Box<dyn Future<Output = ()>>>>,

    #[cfg(not(target_arch = "wasm32"))]
    // Flag set to verify `Condvar` actually woke the executor.
    state: AtomicBool,
}

impl Exec {
    fn new() -> Self {
        #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
        {
            Self {
                mutex: Mutex::new(()),
                cvar: Condvar::new(),
                state: AtomicBool::new(true),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                future: None,
            }
        }
        #[cfg(all(not(target_arch = "wasm32"), not(feature = "std")))]
        {
            Self {
                state: AtomicBool::new(true),
            }
        }
    }
    
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    fn wake(&self) {
        // Wake the task running on a separate thread via CondVar
        if !self.state.compare_and_swap(false, true, Ordering::SeqCst) {
            // We notify the condvar that the value has changed.
            self.cvar.notify_one();
        }
    }

    #[cfg(any(target_arch = "wasm32", not(feature = "std")))]
    fn wake(&self) {
        // Wake the task running on this thread and block until next .await
        // FIXME
    }

    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    #[allow(unsafe_code)]
    fn execute<F: Future<Output = ()>>(&mut self, mut f: F) {
        // Unsafe: f can't move after this, because it is shadowed
        let mut f = unsafe { Pin::new_unchecked(&mut f) };
        // Get a waker and context for this executor.
        let waker = waker(self);
        let context = &mut Context::from_waker(&waker);
        // Run Future to completion.
        while f.as_mut().poll(context).is_pending() {
            // Put the thread to sleep until wake() is called.
            let mut guard = self.mutex.lock().unwrap();
            while !self.state.compare_and_swap(true, false, Ordering::SeqCst) {
                guard = self.cvar.wait(guard).unwrap();
            }
        }
    }
        
    #[cfg(target_arch = "wasm32")]
    fn execute<F: Future<Output = ()>>(&mut self, f: F) {
        // FIXME
    }
    
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "std")))]
    fn execute<F: Future<Output = ()>>(&mut self, mut f: F) {
        // Unsafe: f can't move after this, because it is shadowed
        let mut f = unsafe { Pin::new_unchecked(&mut f) };
        // Get a waker and context for this executor.
        let waker = waker(self);
        let context = &mut Context::from_waker(&waker);
        // Run Future to completion.
        while f.as_mut().poll(context).is_pending() {
            // Put the thread to sleep until wake() is called.
            
            // Fallback implementation, where processor sleep is unavailable
            // (wastes CPU)
            while !self.state.compare_and_swap(true, false, Ordering::SeqCst) { 
            }
        }
    }
}

// When the std library is available, use TLS so that multiple threads can
// lazily initialize an executor.
#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
thread_local! {
    static EXEC: RefCell<Exec> = RefCell::new(Exec::new());
}

// Without std, implement for a single thread.
#[cfg(any(target_arch = "wasm32", not(feature = "std")))]
static EXEC: RefCell<Exec> = RefCell::new(Exec::new());

/// Execute a future.  Similar to the *futures* crate's `executor::block_on()`,
/// except that doesn't necessarily block.
///
/// # Platform-specific behavior
/// On some platforms, this function may block.  On others it may return
/// immediately (this is the case on WASM and embedded systems without an OS).
/// For consistent behavior, always call this as the last function of `main()`
/// or the current thread.
pub fn exec<F: Future<Output = ()>>(f: F) {
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    {
        EXEC.with(|exec| exec.borrow_mut().execute(f));
    }
    #[cfg(any(target_arch = "wasm32", not(feature = "std")))]
    {
        EXEC.borrow_mut().execute(f);
    }
}

// Safe wrapper to create a `Waker`.
#[inline]
#[allow(unsafe_code)]
fn waker(exec: *const Exec) -> Waker {
    const RWVT: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);

    #[inline]
    unsafe fn clone(data: *const ()) -> RawWaker {
        RawWaker::new(data, &RWVT)
    }
    #[inline]
    unsafe fn wake(data: *const ()) {
        let exec: *const Exec = data.cast();
        (*exec).wake();
    }
    #[inline]
    unsafe fn drop(_: *const ()) {}

    unsafe {
        Waker::from_raw(RawWaker::new(exec.cast(), &RWVT))
    }
}
