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
    task::{Context, Poll},
    task::{RawWaker, RawWakerVTable, Waker},
};

#[cfg(not(feature = "std"))]
mod sync {
    use core::sync::atomic::AtomicUsize;

    pub(super) struct Arc<T: Sized> {
        inner: T,
        refcount: AtomicUsize,
    }
}

/// An interrupt handler.
#[allow(unsafe_code)]
pub trait Executor: 'static + Send + Sync + Sized {
    /// Cause `wait_for_event()` to return.
    unsafe fn trigger_event(&'static self);
    /// Blocking wait until an event is triggered with `trigger_event`.  This
    /// function should put the current thread or processor to sleep to save
    /// power consumption.
    unsafe fn wait_for_event(&'static self);

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
    fn block_on<F: Future>(&'static self, mut f: F) -> <F as Future>::Output {
        // unsafe: f can't move after this, because it is shadowed
        let mut f = unsafe { Pin::new_unchecked(&mut f) };

        // Go through the loop each time it wakes up, break when Future ready.
        'executor: loop {
            let waker = waker(self);
            let context = &mut Context::from_waker(&waker);
            match f.as_mut().poll(context) {
                // Go back to waiting for interrupt.
                Poll::Pending => unsafe { self.wait_for_event() },
                Poll::Ready(ret) => break 'executor ret,
            }
        }
    }
}

// Safe wrapper around `Waker` API to get a `Waker` from an `Interrupt`.
#[inline(always)]
#[allow(unsafe_code)]
fn waker<E: Executor>(interrupt: *const E) -> Waker {
    unsafe fn clone<E: Executor>(data: *const ()) -> RawWaker {
        RawWaker::new(data, vtable::<E>())
    }

    unsafe fn wake<E: Executor>(data: *const ()) {
        E::trigger_event(&*(data as *const E));
    }

    unsafe fn drop<E: Executor>(_data: *const ()) {
    }

    unsafe fn vtable<E: Executor>() -> &'static RawWakerVTable {
        &RawWakerVTable::new(clone::<E>, wake::<E>, wake::<E>, drop::<E>)
    }

    unsafe {
        Waker::from_raw(RawWaker::new(interrupt as *const (), vtable::<E>()))
    }
}
