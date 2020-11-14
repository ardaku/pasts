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
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

#[cfg(feature = "std")]
use std::cell::RefCell;

#[cfg(not(any(
    target_arch = "wasm32",
    all(feature = "alloc", not(feature = "std"))
)))]
use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
use std::sync::{Arc, Condvar, Mutex};

#[cfg(any(
    target_arch = "wasm32",
    all(feature = "alloc", not(feature = "std"))
))]
use alloc::{boxed::Box, rc::Rc};

#[cfg(all(
    not(target_arch = "wasm32"),
    all(not(feature = "std"), not(feature = "alloc"))
))]
use core::marker::PhantomData;

#[cfg(any(
    target_arch = "wasm32",
    all(feature = "alloc", not(feature = "std"))
))]
use alloc::vec::Vec;

// Executor data.
struct Exec {
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    // The thread-safe waking mechanism: part 1
    mutex: Mutex<()>,

    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    // The thread-safe waking mechanism: part 2
    cvar: Condvar,

    #[cfg(any(
        target_arch = "wasm32",
        all(feature = "alloc", not(feature = "std"))
    ))]
    // Pinned future.
    tasks: Vec<Pin<Box<dyn Future<Output = ()>>>>,

    #[cfg(not(any(
        target_arch = "wasm32",
        all(feature = "alloc", not(feature = "std"))
    )))]
    // Flag set to verify `Condvar` actually woke the executor.
    state: AtomicBool,
}

impl Exec {
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    fn new() -> Self {
        Self {
            mutex: Mutex::new(()),
            cvar: Condvar::new(),
            state: AtomicBool::new(true),
        }
    }

    #[cfg(any(
        target_arch = "wasm32",
        all(feature = "alloc", not(feature = "std"))
    ))]
    fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    #[cfg(all(
        not(target_arch = "wasm32"),
        all(not(feature = "std"), not(feature = "alloc"))
    ))]
    const fn new() -> Self {
        Self {
            state: AtomicBool::new(true),
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
            // Exit with future output, on future completion, otherwise…
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

    #[cfg(any(
        target_arch = "wasm32",
        all(feature = "alloc", not(feature = "std"))
    ))]
    fn execute<F: Future<Output = ()>>(&mut self, f: F) -> u32
    where
        F: 'static,
    {
        // Get a waker and context for this executor.
        let waker = waker(self);
        let context = &mut Context::from_waker(&waker);
        // Add to task queue
        let task_id = self.tasks.len();
        self.tasks.push(Box::pin(f));
        if let Poll::Ready(output) = self.tasks[task_id].as_mut().poll(context)
        {
            let _ = output;
            todo!();
        }
        task_id as u32
    }

    #[cfg(all(
        not(target_arch = "wasm32"),
        all(not(feature = "std"), not(feature = "alloc"))
    ))]
    #[allow(unsafe_code)]
    fn execute<T, F: Future<Output = T>>(&mut self, mut f: F) -> T {
        // Unsafe: f can't move after this, because it is shadowed
        let mut f = unsafe { Pin::new_unchecked(&mut f) };
        // Get a waker and context for this executor.
        let waker = waker(self);
        let context = &mut Context::from_waker(&waker);
        // Run Future to completion.
        loop {
            // Exit with future output, on future completion, otherwise…
            if let Poll::Ready(value) = f.as_mut().poll(context) {
                break value;
            }
            // Put the thread to sleep until wake() is called.

            // Fallback implementation, where processor sleep is unavailable
            // (wastes CPU)
            while !self.state.compare_and_swap(true, false, Ordering::SeqCst) {}
        }
    }
}

// When the std library is available, use TLS so that multiple threads can
// lazily initialize an executor.
#[cfg(all(feature = "std"))]
thread_local! {
    static EXEC: RefCell<Exec> = RefCell::new(Exec::new());
}

// Without std, implement for a single thread.
#[cfg(not(feature = "std"))]
static mut EXEC: Exec = Exec::new();

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
/// pasts::spawn(async_main);
/// ```
pub fn spawn<T, F: Future<Output = T>, G: Fn() -> F>(g: G) -> JoinHandle<T>
where
    T: 'static + Send + Unpin,
    G: 'static + Send,
{
    // Can start tasks on their own threads.
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

    // Can spawn singular task only (block on it).
    //
    // FIXME: Maybe can spawn multiple without allocator?
    #[cfg(all(
        not(target_arch = "wasm32"),
        all(not(feature = "std"), not(feature = "alloc"))
    ))]
    #[allow(unsafe_code)]
    {
        let _output = unsafe { EXEC.execute(g()) };
        JoinHandle { waker: PhantomData }
    }

    // Can allocate task queue.
    #[cfg(any(target_arch = "wasm32",))]
    {
        let waker = Rc::new((None, None));
        let mut waker_b = waker.clone();
        JoinHandle {
            handle: EXEC.with(|exec| {
                exec.borrow_mut().execute(async move {
                    let output = g().await;
                    Rc::get_mut(&mut waker_b).unwrap().0 = Some(output);
                })
            }),
            waker,
        }
    }

    #[cfg(all(
        not(target_arch = "wasm32"),
        feature = "alloc",
        not(feature = "std")
    ))]
    #[allow(unsafe_code)]
    {
        let waker = Rc::new((None, None));
        let mut waker_b = waker.clone();
        JoinHandle {
            handle: unsafe {
                EXEC.execute(async move {
                    let output = g().await;
                    Rc::get_mut(&mut waker_b).unwrap().0 = Some(output);
                })
            },
            waker,
        }
    }
}

/// An owned permission to join on a task (`.await` on its termination).
#[derive(Debug)]
pub struct JoinHandle<T>
where
    T: Unpin,
{
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    waker: Arc<Mutex<(Option<T>, Option<Waker>)>>,

    #[cfg(any(
        target_arch = "wasm32",
        all(feature = "alloc", not(feature = "std"))
    ))]
    waker: Rc<(Option<T>, Option<Waker>)>,

    #[cfg(all(
        not(target_arch = "wasm32"),
        all(not(feature = "std"), not(feature = "alloc"))
    ))]
    waker: PhantomData<T>,

    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    handle: Option<std::thread::JoinHandle<()>>,

    #[cfg(any(
        target_arch = "wasm32",
        all(feature = "alloc", not(feature = "std"))
    ))]
    handle: u32,
}

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
impl<T: Unpin> Drop for JoinHandle<T> {
    fn drop(&mut self) {
        self.handle.take().unwrap().join().unwrap();
    }
}

#[cfg(any(target_arch = "wasm32", feature = "alloc"))]
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
            let this = self.get_mut();
            if let Some(output) = Rc::get_mut(&mut this.waker).unwrap().0.take()
            {
                Poll::Ready(output)
            } else {
                Rc::get_mut(&mut this.waker).unwrap().1 =
                    Some(cx.waker().clone());
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
