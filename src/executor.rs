// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

#[cfg(not(target_arch = "wasm32"))]
use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
use std::sync::{Condvar, Mutex, Arc};

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
            Self { future: None }
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
    fn execute<T, F: Future<Output = T>>(&mut self, mut f: F) -> T {
        // Unsafe: f can't move after this, because it is shadowed
        let mut f = unsafe { Pin::new_unchecked(&mut f) };
        // Get a waker and context for this executor.
        let waker = waker(self);
        let context = &mut Context::from_waker(&waker);
        // Run Future to completion.
        loop {
            // 
            if let Poll::Ready(value) = f.as_mut().poll(context) {
                break value;
            }
            // Put the thread to sleep until wake() is called.
            let mut guard = self.mutex.lock().unwrap();
            while !self.state.compare_and_swap(true, false, Ordering::SeqCst) {
                guard = self.cvar.wait(guard).unwrap();
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn execute<T, F: Future<Output = T>>(&mut self, mut f: F) -> T {
        // FIXME
    }

    #[cfg(all(not(target_arch = "wasm32"), not(feature = "std")))]
    fn execute<T, F: Future<Output = T>>(&mut self, mut f: F) -> T {
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
            while !self.state.compare_and_swap(true, false, Ordering::SeqCst) {}
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

/// Execute a future by spawning an asynchronous task.
///
/// On multi-threaded systems, this will start a new thread.  Similar to
/// `futures::executor::block_on()`, except that doesn't block.  Similar to
/// `std::thread::spawn()`, except that tasks don't detach, and will join on
/// `Drop` (except on WASM, where the program continues after main() exits).
///
/// # Example
/// ```rust
/// async fn async_main() {
///     /* your code here */
/// }
///
/// fn main() {
///     pasts::spawn(async_main);
/// }
/// ```
pub fn spawn<T, F: Future<Output = T>, G: Fn() -> F>(g: G) -> JoinHandle<T>
    where T: 'static + Send + Unpin, G: 'static + Send
{
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    {
        let waker = Arc::new(Mutex::new((None, None)));
        JoinHandle {
            waker: waker.clone(),
            handle: Some(std::thread::spawn(move || {
                let output = EXEC.with(|exec| exec.borrow_mut().execute(g()));
                let mut waker = waker.lock().unwrap();
                waker.0 = Some(output);
                if let Some(waker) = waker.1.take() {
                    waker.wake();
                }
            })),
        }
    }

    #[cfg(any(target_arch = "wasm32", not(feature = "std")))]
    {
        EXEC.borrow_mut().execute(f);
    }
}

/// An owned permission to join on a task (`.await` on its termination).
#[derive(Debug)]
pub struct JoinHandle<T> where T: Unpin {
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    waker: Arc<Mutex<(Option<T>, Option<Waker>)>>,

    #[cfg(any(target_arch = "wasm32", not(feature = "std")))]
    waker: (Option<T>, Option<Waker>),

    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    handle: Option<std::thread::JoinHandle<()>>,

    #[cfg(any(target_arch = "wasm32", not(feature = "std")))]
    handle: u32,
}

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
impl<T: Unpin> Drop for JoinHandle<T> {
    fn drop(&mut self) {
        self.handle.take().unwrap().join().unwrap();
    }
}

impl<T: Unpin> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
        {
            let mut waker = self.waker.lock().unwrap();
            if let Some(output) = waker.0.take() {
                Poll::Ready(output)
            } else {
                waker.1 = Some(cx.waker().clone());
                Poll::Pending
            }
        }
        
        #[cfg(any(target_arch = "wasm32", not(feature = "std")))]
        {
            if let Some(output) = self.waker.0.take() {
                Poll::Ready(output)
            } else {
                self.waker.1 = Some(cx.waker().clone());
                Poll::Pending
            }
        }
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

    unsafe { Waker::from_raw(RawWaker::new(exec.cast(), &RWVT)) }
}
