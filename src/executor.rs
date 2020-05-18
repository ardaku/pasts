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

/// An executor for `Future`s.
#[allow(unsafe_code)]
pub trait Executor: 'static + Send + Sync + Sized {
    /// Cause `wait_for_event()` to return.
    ///
    /// # Safety
    /// This method is marked `unsafe` because it must only be called from a
    /// `Waker`.  This is guaranteed by the `block_on()` method.
    unsafe fn trigger_event(&self);
    /// Blocking wait until an event is triggered with `trigger_event`.  This
    /// function should put the current thread or processor to sleep to save
    /// power consumption.
    ///
    /// # Safety
    /// This function should only be called by one executor.  On the first call
    /// to this method, all following calls to `is_used()` should return `true`.
    /// This method is marked `unsafe` because only one thread and one executor
    /// can call it (ever!).  This is guaranteed by the `block_on()` method.
    unsafe fn wait_for_event(&self);
    /// Should return true if `wait_for_event` has been called, false otherwise.
    fn is_used(&self) -> bool;

    /// Run a future to completion on the current thread.  This will cause the
    /// current thread to block.
    #[allow(unsafe_code)]
    #[inline]
    fn block_on<F: Future>(&'static self, mut f: F) -> <F as Future>::Output {
        if self.is_used() {
            panic!("Can't reuse an executor!");
        }

        // unsafe: f can't move after this, because it is shadowed
        let mut f = unsafe { Pin::new_unchecked(&mut f) };

        // Go through the loop each time it wakes up, break when Future ready.
        let waker = waker(self);
        let context = &mut Context::from_waker(&waker);
        'executor: loop {
            unsafe { self.wait_for_event() };
            if let Poll::Ready(ret) = f.as_mut().poll(context) {
                break 'executor ret;
            }
        }
    }
}

// Safe wrapper around `Waker` API to get a `Waker` from an `Interrupt`.
#[inline]
#[allow(unsafe_code)]
fn waker<E: Executor>(interrupt: *const E) -> Waker {
    #[inline]
    unsafe fn clone<E: Executor>(data: *const ()) -> RawWaker {
        RawWaker::new(
            data,
            &RawWakerVTable::new(clone::<E>, wake::<E>, wake::<E>, drop::<E>),
        )
    }

    #[inline]
    unsafe fn wake<E: Executor>(data: *const ()) {
        E::trigger_event(&*(data as *const E));
    }

    #[inline]
    unsafe fn drop<E: Executor>(_data: *const ()) {}

    unsafe {
        Waker::from_raw(RawWaker::new(
            interrupt as *const (),
            &RawWakerVTable::new(clone::<E>, wake::<E>, wake::<E>, drop::<E>),
        ))
    }
}
