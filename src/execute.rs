use crate::_pasts_hide::stn::{
    future::Future,
    task::{Context, Poll},
    task::{RawWaker, RawWakerVTable, Waker},
};

/// An interrupt handler.
pub trait Interrupt: Send + Sync + Sized {
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
    fn block_on<F: Future>(mut f: F) -> <F as Future>::Output {
        let mut f = crate::tasks::new_pin(&mut f);

        let task: Self = Interrupt::new();

        // Check for any futures that are ready
        loop {
            let waker = waker(&task);
            let context = &mut Context::from_waker(&waker);
            match f.as_mut().poll(context) {
                // Go back to waiting for interrupt.
                Poll::Pending => task.wait_for(),
                Poll::Ready(ret) => break ret,
            }
        }
    }
}

// Safe wrapper around `Waker` API to get a `Waker` from an `Interrupt`.
#[inline(always)]
#[allow(unsafe_code)]
fn waker<I: Interrupt>(interrupt: *const I) -> Waker {
    unsafe fn clone<I: Interrupt>(data: *const ()) -> RawWaker {
        RawWaker::new(data, vtable::<I>())
    }

    unsafe fn wake<I: Interrupt>(data: *const ()) {
        ref_wake::<I>(data)
    }

    unsafe fn ref_wake<I: Interrupt>(data: *const ()) {
        I::interrupt(&*(data as *const I));
    }

    unsafe fn drop<I: Interrupt>(_data: *const ()) {}

    unsafe fn vtable<I: Interrupt>() -> &'static RawWakerVTable {
        &RawWakerVTable::new(clone::<I>, wake::<I>, ref_wake::<I>, drop::<I>)
    }

    unsafe {
        Waker::from_raw(RawWaker::new(interrupt as *const (), vtable::<I>()))
    }
}
