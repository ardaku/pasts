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

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
use std::cell::RefCell;

use core::task::{Context, RawWaker, RawWakerVTable, Waker};

use crate::exec::Exec;

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
    ($(let $x:ident = $y:expr);* $(;)?) => {
        $(
            let mut $x = $y;
            #[allow(unused_mut, unused_qualifications)]
            let mut $x = unsafe {
                core::pin::Pin::<&mut dyn core::future::Future<Output = _>>
                    ::new_unchecked(&mut $x)
            };
        )*
    };
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

// When the std library is available, use TLS so that multiple threads can
// lazily initialize an executor.
#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
thread_local! {
    static EXEC: RefCell<Exec> = RefCell::new(Exec::new());
}

// Without std, assume no threads, and use "fake" TLS.
#[cfg(any(target_arch = "wasm32", not(feature = "std")))]
static mut EXEC: Option<Exec> = None;

// Get a reference to the thread local, or if there are no threads, the global
// static.
#[inline]
pub(super) fn exec<F, T>(f: F) -> T
where
    F: FnOnce(&mut Exec) -> T,
{
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    {
        EXEC.with(|exec| f(&mut exec.borrow_mut()))
    }

    #[cfg(any(target_arch = "wasm32", not(feature = "std")))]
    // unsafe: safe because there are no threads.
    unsafe {
        if let Some(ref mut exec) = EXEC.as_mut() {
            f(exec)
        } else {
            EXEC = Some(Exec::new());
            f(EXEC.as_mut().unwrap())
        }
    }
}
