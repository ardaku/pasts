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
    task::{Context, Poll},
    task::{RawWaker, RawWakerVTable, Waker},
    pin::Pin,
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
    #[allow(unsafe_code)]
    fn block_on<F: Future>(mut f: F) -> <F as Future>::Output {
        let task: Self = Interrupt::new();

        // unsafe: f can't move after this, because it is shadowed
        let mut f = unsafe { Pin::new_unchecked(&mut f) };

        // Go through the loop each time it wakes up, break when Future ready.
        'executor: loop {
            let waker = waker(&task);
            let context = &mut Context::from_waker(&waker);
            match f.as_mut().poll(context) {
                // Go back to waiting for interrupt.
                Poll::Pending => task.wait_for(),
                Poll::Ready(ret) => break 'executor ret,
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
