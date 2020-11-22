// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//
//! This file contains functions and macros that require unsafe code to work.
//! The rest of the libary should be unsafe-free.

#![allow(unsafe_code)]

use core::{
    future::Future,
    pin::Pin,
    task::{Context, RawWaker, RawWakerVTable, Waker},
};

use crate::exec::Exec;

/// A pinned future trait object.
pub type Task<'a, T> = Pin<&'a mut dyn Future<Output = T>>;

/// Create a future that waits on multiple futures and returns their results as
/// a tuple.
///
/// ```rust
/// use pasts::prelude::*;
///
/// async fn async_main() {
///     task! {
///         let a = async { "Hello, World!" };
///         let b = async { 15u32 };
///         let c = async { 'c' };
///     };
///     assert_eq!(("Hello, World!", 15u32, 'c'), join!(a, b, c));
/// }
///
/// exec!(async_main());
/// ```
#[macro_export]
macro_rules! join {
    // FIXME: Make this code simpler and easier to understand.
    ($($future:ident),* $(,)?) => {{
        use core::{task::{Poll, Context}, mem::MaybeUninit, pin::Pin, future::Future};
        struct MaybeFuture<'a, T, F: Future<Output = T>>(Option<Pin<&'a mut F>>);
        impl <T, F: Future<Output = T>> Future for MaybeFuture<'_, T, F> {
            type Output = T;
            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
                let mut this = self.as_mut();
                if let Some(future) = this.0.as_mut() {
                    match future.as_mut().poll(cx) {
                        Poll::Ready(t) => {
                            this.0 = None;
                            Poll::Ready(t)
                        },
                        a => a,
                    }
                } else {
                    Poll::Pending
                }
            }
        }
        let mut count = 0usize;
        $(
            let mut $future = MaybeFuture(Some(Pin::new(&mut $future)));
            let mut $future = (Pin::new(&mut $future), MaybeUninit::uninit());
            count += 1;
        )*
        for _ in 0..count {
            struct __Pasts_Selector<'a, T> {
                closure: &'a mut dyn FnMut(&mut Context<'_>) -> Poll<T>,
            }
            impl<'a, T> Future for __Pasts_Selector<'a, T> {
                type Output = T;
                fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
                    (self.get_mut().closure)(cx)
                }
            }
            __Pasts_Selector { closure: &mut |__pasts_cx: &mut Context<'_>| {
                $(
                    match $future.0.as_mut().poll(__pasts_cx) {
                        Poll::Ready(pattern) => {
                            $future.1 = MaybeUninit::new(pattern);
                            return Poll::Ready(());
                        }
                        Poll::Pending => {}
                    }
                )*
                Poll::Pending
            } }.await
        }
        unsafe {
            ($($future.1.assume_init()),*)
        }
    }};
}

/// Create future trait object(s) that implement [`Unpin`](std::marker::Unpin).
///
/// ```rust
/// use pasts::prelude::*;
///
/// async fn bar() { }
///
/// task!(let task_name = async { "Hello, world" });
/// task! {
///     let foo = async {};
///     let bar = bar();
/// }
/// ```
#[macro_export]
macro_rules! task {
    // unsafe: safe because once value is moved and then shadowed, one can't
    // directly access anymore.
    ($(let $x:ident = $y:expr);* $(;)?) => { $(
        let mut $x = $y;
        #[allow(unused_mut, unused_qualifications)]
        let mut $x = unsafe {
            core::pin::Pin::<&mut dyn core::future::Future<Output = _>>
                ::new_unchecked(&mut $x)
        };
    )* };
    ($x:ident) => {
        core::pin::Pin::<&mut dyn core::future::Future<Output = _>>
            ::new(&mut $x)
    };
}

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
