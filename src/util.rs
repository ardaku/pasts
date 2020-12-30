// Copyright Jeron Aldaron Lau 2019-2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.
//
//! This file contains functions and macros that require unsafe code to work.
//! The rest of the libary should be unsafe-free.

use core::task::{Context, RawWaker, RawWakerVTable, Waker};

use crate::exec::Exec;

/// Create a future that waits on multiple futures concurrently and returns
/// their results as a tuple.
///
/// ```rust
/// use pasts::join;
///
/// async fn async_main() {
///     let a = async { "Hello, World!" };
///     let b = async { 15u32 };
///     let c = async { 'c' };
///     assert_eq!(("Hello, World!", 15u32, 'c'), join!(a, b, c));
/// }
///
/// pasts::block_on(async_main());
/// ```
#[macro_export]
macro_rules! join {
    // FIXME: Make this code simpler and easier to understand.
    ($($future:ident),* $(,)?) => {{
        use core::{
            task::{Poll, Context}, mem::MaybeUninit, pin::Pin, future::Future
        };

        // Move and Pin all the futures passed in.
        $(
            let mut $future = $future;
            // unsafe: safe because once value is moved and then shadowed, one
            // can't directly access anymore.
            let mut $future = unsafe {
                Pin::<&mut dyn Future<Output = _>>::new_unchecked(&mut $future)
            };
        )*

        // Join Algorithm
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

// Create a `Waker`.
//
// unsafe: Safe because `Waker`/`Context` can't outlive `Exec`.
#[inline]
#[allow(unsafe_code)]
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
#[allow(unsafe_code)]
// unsafe: sound because threads can't happen on targets with no threads.
pub(crate) fn exec() -> &'static mut Exec {
    unsafe {
        EXEC = core::mem::MaybeUninit::new(Exec::new());
        &mut *EXEC.as_mut_ptr()
    }
}
