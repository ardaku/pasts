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

use std::task::{Waker, Context, RawWaker, RawWakerVTable};

/// Create a future trait objects that implement `Unpin`.
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
        #[allow(unused_mut)]
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
pub(super) fn waker<F, T>(exec: &crate::exec::Exec, f: F) -> T
    where F: FnOnce(&mut Context<'_>) -> T
{
    let exec: *const crate::exec::Exec = exec;
    const RWVT: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);

    #[inline]
    unsafe fn clone(data: *const ()) -> RawWaker {
        RawWaker::new(data, &RWVT)
    }
    #[inline]
    unsafe fn wake(data: *const ()) {
        let exec: *const crate::exec::Exec = data.cast();
        (*exec).wake();
    }
    #[inline]
    unsafe fn drop(_: *const ()) {}

    let waker = unsafe { Waker::from_raw(RawWaker::new(exec.cast(), &RWVT)) };
    f(&mut Context::from_waker(&waker))
}
