mod wake;

use self::wake::Wake;

use crate::{
    _pasts_hide::stn::{
        future::Future,
        task::{Context, Poll},
    },
    let_pin,
};

/// An interrupt handler.
pub trait Interrupt {
    /// Initialize the shared data for the interrupt.
    fn new() -> Self;
    /// Interrupt blocking to wake up.
    fn interrupt(&self);
    /// Blocking wait for interrupt, if `Poll::Ready` then stop blocking.
    fn wait_for(&self);

    /// Run a future to completion on the current thread.  This will cause the
    /// current thread to block.
    ///
    /// ```rust
    /// let ret = <pasts::CondvarInterrupt as pasts::Interrupt>::block_on(
    ///     async {
    ///         /* Do some work, calling .await on futures */
    ///         "Complete!"
    ///     }
    /// );
    /// assert_eq!(ret, "Complete!");
    /// ```
    fn block_on<F: Future/*, I: Interrupt + Send + Sync*/>(
        f: F,
    ) -> <F as Future>::Output
        where Self: Send + Sync + Sized
    {
        pub struct FutureTask<I: Interrupt>(I);

        impl<I: Interrupt + Send + Sync> Wake for FutureTask<I> {
            fn wake_up(&self) {
                self.0.interrupt();
            }
        }

        let_pin! { future_one = f; }

        let task = FutureTask::<Self>(Interrupt::new());

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
}
