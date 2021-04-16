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
//! This file contains functions and that require unsafe code to work.
//! The rest of the libary should be unsafe-free.

#![allow(unsafe_code)]

use core::task::{Context, RawWaker, RawWakerVTable, Waker};

use crate::exec::Exec;

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
    fn clone(data: *const ()) -> RawWaker {
        RawWaker::new(data, &RWVT)
    }
    #[inline]
    unsafe fn wake(data: *const ()) {
        let exec: *const Exec = data.cast();
        (*exec).wake();
    }
    #[inline]
    fn drop(_: *const ()) {}

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
