// Pasts
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).
//
//! This file contains functions and macros that require unsafe code to work.
//! The rest of the libary should be unsafe-free.

#![allow(unsafe_code)]

use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::Poll;
use core::task::{Context, RawWaker, RawWakerVTable, Waker};

use crate::{exec::Exec, past::Past, race::Stateful};

// Create a `Waker`.
//
// unsafe: Safe because `Waker`/`Context` can't outlive `Exec`.
#[inline]
pub(super) fn waker<F, T>(exec: &Exec, f: F) -> T
where
    F: FnOnce(&mut Context<'_>) -> T,
{
    let exec: *const Exec = exec;
    const RWVT: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);

    #[inline]
    unsafe fn clone(data: *const ()) -> RawWaker {
        RawWaker::new(data, &RWVT)
    }
    #[inline]
    unsafe fn wake(data: *const ()) {
        let exec: *const Exec = data.cast();
        (*exec).wake();
    }
    #[inline]
    unsafe fn drop(_: *const ()) {}

    let waker = unsafe { Waker::from_raw(RawWaker::new(exec.cast(), &RWVT)) };
    f(&mut Context::from_waker(&waker))
}

#[cfg(any(target_arch = "wasm32", not(feature = "std")))]
static mut EXEC: core::mem::MaybeUninit<Exec> =
    core::mem::MaybeUninit::<Exec>::uninit();

#[cfg(any(target_arch = "wasm32", not(feature = "std")))]
// unsafe: sound because threads can't happen on targets with no threads.
pub(crate) fn exec() -> &'static mut Exec {
    unsafe {
        EXEC = core::mem::MaybeUninit::new(Exec::new());
        &mut *EXEC.as_mut_ptr()
    }
}

#[allow(missing_debug_implementations)]
pub struct MultiFuture<S, F, L, G, U>
where
    F: Future<Output = U> + Unpin,
    G: Future<Output = L> + Stateful<S> + Unpin,
{
    pub(crate) future: *mut F,
    pub(crate) other: G,
    pub(crate) translator: fn(&mut S, U) -> L,
}

impl<S, F, L, G, U> Stateful<S> for MultiFuture<S, F, L, G, U>
where
    F: Future<Output = U> + Unpin,
    G: Future<Output = L> + Stateful<S> + Unpin,
{
    fn state(&mut self) -> *mut S {
        self.other.state()
    }
}

impl<S, F, L, G, U> Future for MultiFuture<S, F, L, G, U>
where
    F: Future<Output = U> + Unpin,
    G: Future<Output = L> + Stateful<S> + Unpin,
{
    type Output = L;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        match Pin::new(&mut self.other).poll(cx) {
            Poll::Pending => {
                match Pin::new(unsafe { &mut *self.future }).poll(cx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(output) => Poll::Ready((self.translator)(
                        unsafe { &mut *self.state() },
                        output,
                    )),
                }
            }
            x => x,
        }
    }
}

#[repr(transparent)]
#[allow(missing_debug_implementations)]
pub struct PastFuture<U, P: Past<U>>(P, PhantomData<*mut U>);

impl<U, P: Past<U>> PastFuture<U, P> {
    #[allow(trivial_casts)] // Not sure why it thinks it's trivial, is needed.
    pub(crate) fn with(from: &mut P) -> &mut Self {
        unsafe { &mut *(from as *mut _ as *mut Self) }
    }
}

impl<U, P: Past<U>> Future for PastFuture<U, P> {
    type Output = U;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}
