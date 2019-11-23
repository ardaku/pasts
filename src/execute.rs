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
/// assert_eq!("Complete!", ret);
/// ```
pub fn block_on<F: Future>(f: F) -> <F as Future>::Output {
    static mut FUTURE_CONDVARS: [AtomicBool; 1] = [AtomicBool::new(true)];

    pub struct FutureZeroTask();

    impl Wake for FutureZeroTask {
        unsafe fn wake_up() {
            FUTURE_CONDVARS[0].store(true, Ordering::Relaxed);
        }
    }

    let_pin! {
        future_one = f;
        task = FutureZeroTask();
    };

    // Check for any futures that are ready

    loop {
        if unsafe { FUTURE_CONDVARS[0].load(Ordering::Relaxed) } {
            // This runs whenever woke.
            let task = unsafe { FutureZeroTask::into_waker(&*task) };
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
