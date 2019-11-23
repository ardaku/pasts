//! Minimal and simpler alternative to the futures crate.

#![no_std]
#![warn(missing_docs)]

mod wake;

pub use wake::{Wake};

/// Pin a variable to a location in the stack.
///
/// ```rust
/// pasts::let_pin! {
///     var = "Hello, world";
/// };
/// let _: core::pin::Pin<&mut &str> = var;
/// ```
#[macro_export]
macro_rules! let_pin {
    ($($x:ident = $y:expr);* $(;)?) => { $(
        // Force move.
        let mut $x = $y;
        // Shadow to prevent future use.
        #[allow(unused_mut)]
        let mut $x = unsafe { core::pin::Pin::new_unchecked(&mut $x) };
    )* }
}

/// Run a future to completion on the current thread.  This will cause the
/// current thread to block.
pub fn block_on<F: core::future::Future>(f: F) -> <F as core::future::Future>::Output {
    use core::{
        sync::atomic::{AtomicBool, Ordering},
        task::{Context, Poll},
    };

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
