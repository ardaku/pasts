mod wake;

use self::wake::Wake;

use crate::{
    _pasts_hide::stn::{
        future::Future,
        task::{Context, Poll},
    },
    let_pin,
};

#[cfg(feature = "std")]
use crate::_pasts_hide::stn::sync::{Condvar, Mutex};

#[cfg(not(feature = "std"))]
use crate::_pasts_hide::stn::sync::atomic::{AtomicBool, Ordering};

/// An interrupt handler.
pub trait Interrupt {
    /// Interrupt blocking to wake up.
    fn interrupt(&self);
    /// Blocking wait for interrupt, if `Poll::Ready` then stop blocking.
    fn wait_for(&self);
}

/// Blocking wait until interrupt using wake handler.
pub fn block_until<F: Future, I: Interrupt + Send + Sync>(f: F, i: I)
    -> <F as Future>::Output
{
    pub struct FutureTask<I: Interrupt>(I);

    impl<I: Interrupt + Send + Sync> Wake for FutureTask<I> {
        fn wake_up(&self) {
            self.0.interrupt();
        }
    }

    let_pin! { future_one = f; }

    let task = FutureTask(i);

    // Check for any futures that are ready
    loop {
        let waker = FutureTask::into_waker(&task);
        let context = &mut Context::from_waker(&waker);
        match future_one.as_mut().poll(context) {
            // Go back to waiting for interrupt.
            Poll::Pending => task.0.wait_for(),
            Poll::Ready(ret) => break ret,
        }
    }
}

/// Run a future to completion on the current thread.  This will cause the
/// current thread to block.
///
/// ```rust
/// let ret = pasts::block_on(
///     async {
///         /* Do some work, calling .await on futures */
///         "Complete!"
///     }
/// );
/// assert_eq!(ret, "Complete!");
/// ```
pub fn block_on<F: Future>(f: F) -> <F as Future>::Output {
    #[cfg(feature = "std")]
    pub struct FutureTask(Mutex<()>, Condvar);

    #[cfg(not(feature = "std"))]
    pub struct FutureTask(AtomicBool);

    impl Wake for FutureTask {
        #[cfg(feature = "std")]
        fn wake_up(&self) {
            self.1.notify_one();
        }

        #[cfg(not(feature = "std"))]
        fn wake_up(&self) {
            self.0.store(true, Ordering::Relaxed);
        }
    }

    let_pin! { future_one = f; }

    #[cfg(not(feature = "std"))]
    let_pin! { task = FutureTask(AtomicBool::new(true)); };
    // Check for any futures that are ready
    #[cfg(not(feature = "std"))]
    loop {
        if task.0.load(Ordering::Relaxed) {
            // This runs whenever woke.
            let waker = FutureTask::into_waker(&*task);
            let context = &mut Context::from_waker(&waker);
            match future_one.as_mut().poll(context) {
                // Go back to "sleep".
                Poll::Pending => task.0.store(false, Ordering::Relaxed),
                Poll::Ready(ret) => break ret,
            }
        }
    }

    #[cfg(feature = "std")]
    let_pin! { task = FutureTask(Mutex::new(()), Condvar::new()); };
    #[cfg(feature = "std")]
    let mut guard = task.0.lock().unwrap();
    #[cfg(feature = "std")]
    loop {
        // This runs whenever woke.
        let waker = FutureTask::into_waker(&*task);
        let context = &mut Context::from_waker(&waker);
        match future_one.as_mut().poll(context) {
            Poll::Pending => { /* continue, going back to sleep */ }
            Poll::Ready(ret) => break ret,
        }

        // Save some processing, by putting the thread to sleep.
        guard = task.1.wait(guard).unwrap();
    }
}
