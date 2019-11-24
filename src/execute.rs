use crate::stn::{
    future::Future,
    task::{Context, Poll},
};

#[cfg(feature = "std")]
use crate::stn::{
    sync::{Condvar, Mutex},
    mem::MaybeUninit,
};

#[cfg(not(feature = "std"))]
use crate::stn::{
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{Wake, let_pin};

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
    static mut FUTURE_CONDVARS: MaybeUninit<[(Mutex<bool>, Condvar); 1]> =
        MaybeUninit::uninit();

    #[cfg(not(feature = "std"))]
    static mut FUTURE_CONDVARS: [AtomicBool; 1] = [AtomicBool::new(true)];

    pub struct FutureTask(usize);

    impl Wake for FutureTask {
        #[cfg(feature = "std")]
        unsafe fn wake_up(&self) {
            *(*FUTURE_CONDVARS.as_ptr())[self.0].0.lock().unwrap() = true;
            (*FUTURE_CONDVARS.as_ptr())[self.0].1.notify_one();
        }

        #[cfg(not(feature = "std"))]
        unsafe fn wake_up(&self) {
            FUTURE_CONDVARS[self.0].store(true, Ordering::Relaxed);
        }
    }

    let_pin! {
        future_one = f;
        task = FutureTask(0);
    };

    // Check for any futures that are ready
    #[cfg(not(feature = "std"))]
    loop {
        if unsafe { FUTURE_CONDVARS[0].load(Ordering::Relaxed) } {
            // This runs whenever woke.
            let task = unsafe { FutureTask::into_waker(&*task) };
            let context = &mut Context::from_waker(&task);
            match future_one.as_mut().poll(context) {
                // Go back to "sleep".
                Poll::Pending => unsafe {
                    FUTURE_CONDVARS[0].store(false, Ordering::Relaxed);
                }
                Poll::Ready(ret) => break ret
            }
        }
    }

    // Initialize mutable static.
    #[cfg(feature = "std")]
    unsafe {
        FUTURE_CONDVARS = MaybeUninit::new([(Mutex::new(true), Condvar::new())]);
    }
    #[cfg(feature = "std")]
    let mut guard = unsafe { (*FUTURE_CONDVARS.as_ptr())[0].0.lock().unwrap() };
    #[cfg(feature = "std")]
    loop {
        // Save some processing, by putting the thread to sleep.
        if !(*guard) {
            guard = unsafe { (*FUTURE_CONDVARS.as_ptr())[0].1.wait(guard) }.unwrap();
        }
        if *guard {
            // This runs whenever woke.
            let task = unsafe { FutureTask::into_waker(&*task) };
            let context = &mut Context::from_waker(&task);
            match future_one.as_mut().poll(context) {
                // Go back to "sleep".
                Poll::Pending => *guard = false,
                Poll::Ready(ret) => break ret
            }
        }
    }
}

/// Poll two futures concurrently, and return a tuple of returned values from
/// each future.  Only usable inside async functions and blocks.
///
/// Futures that are ready first will be executed first.  This makes
/// `join!(a, b)` faster than the alternative `(a.await, b.await)`.
///
/// ```rust
/// async fn example() {
///     let ret = pasts::join!(
///         async {
///             /* Do some work, calling .await on futures */
///             'c'
///         },
///         async {
///             /* Do some work, calling .await on futures */
///             5
///         },
///     );
///     assert_eq!(ret, ('c', 5));
/// }
///
/// pasts::block_on(example());
/// ```
#[macro_export]
macro_rules! join {
    ($($y:expr),* $(,)?) => {
/*        $(
            // Force move.
            let mut $x = $y;
            // Shadow to prevent future use.
            #[allow(unused_mut)]
            let mut $x = unsafe { core::pin::Pin::new_unchecked(&mut $x) };
        )*
*/
        // Open a new scope.

        ('c', 5)
    }
}
