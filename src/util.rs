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
