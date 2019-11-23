use core::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll},
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
    static mut FUTURE_CONDVARS: [AtomicBool; 1] = [AtomicBool::new(true)];

    pub struct FutureTask(usize);

    impl Wake for FutureTask {
        unsafe fn wake_up(&self) {
            FUTURE_CONDVARS[self.0].store(true, Ordering::Relaxed);
        }
    }

    let_pin! {
        future_one = f;
        task = FutureTask(0);
    };

    // Check for any futures that are ready
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
        $(
/*            // Force move.
            let mut $x = $y;
            // Shadow to prevent future use.
            #[allow(unused_mut)]
            let mut $x = unsafe { core::pin::Pin::new_unchecked(&mut $x) };*/
        )*
    }
}
