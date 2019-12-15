mod wake;

use self::wake::Wake;

use crate::{
    _pasts_hide::{new_pin, stn::{
        future::Future,
        task::{Context, Poll},
    }},
};

/// An interrupt handler.
pub trait Interrupt {
    /// Initialize the shared data for the interrupt.
    fn new() -> Self;
    /// Interrupt blocking to wake up.
    fn interrupt(&self);
    /// Blocking wait until interrupt.
    fn wait_for(&self);

    /// Run a future to completion on the current thread.  This will cause the
    /// current thread to block.
    ///
    /// ```rust
    /// use pasts::prelude::*;
    ///
    /// let ret = pasts::ThreadInterrupt::block_on(
    ///     async {
    ///         /* Do some work, calling .await on futures */
    ///         "Complete!"
    ///     }
    /// );
    /// assert_eq!(ret, "Complete!");
    /// ```
    fn block_on<F: Future>(mut f: F) -> <F as Future>::Output
    where
        Self: Send + Sync + Sized,
    {
        pub struct FutureTask<I: Interrupt>(I);

        impl<I: Interrupt + Send + Sync> Wake for FutureTask<I> {
            fn wake_up(&self) {
                self.0.interrupt();
            }
        }

        let mut f = new_pin(&mut f);

        let task = FutureTask::<Self>(Interrupt::new());

        // Check for any futures that are ready
        loop {
            let waker = FutureTask::into_waker(&task);
            let context = &mut Context::from_waker(&waker);
            match f.as_mut().poll(context) {
                // Go back to waiting for interrupt.
                Poll::Pending => task.0.wait_for(),
                Poll::Ready(ret) => break ret,
            }
        }
    }
}
