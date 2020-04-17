// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::{future::Future, pin::Pin, task::Context, task::Poll};

pub enum SelectFuture<'a, 'b, T> {
    Future(&'b mut [&'a mut dyn Future<Output = T>]),
    OptFuture(&'b mut [Option<&'a mut dyn Future<Output = T>>]),
}

impl<T> core::fmt::Debug for SelectFuture<'_, '_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Future(_) => write!(f, "Future"),
            Self::OptFuture(_) => write!(f, "OptFuture"),
        }
    }
}

impl<T> Future for SelectFuture<'_, '_, T> {
    type Output = (usize, T);

    // unsafe: This let's this future create `Pin`s from the slices it has a
    // unique reference to.  This is safe because `SelectFuture` never calls
    // `mem::swap()` and when `SelectFuture` drops it's no longer necessary
    // that the memory remain pinned because it's not being polled anymore.
    #[allow(unsafe_code)]
    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let mut task_id = 0;
        match *self {
            SelectFuture::Future(ref mut tasks) => {
                for task in tasks.iter().map(|task| {
                    let mut pin_fut =
                        unsafe { Pin::new_unchecked(std::ptr::read(task)) };
                    let ret = pin_fut.as_mut().poll(cx);
                    std::mem::forget(pin_fut);
                    ret
                }) {
                    match task {
                        Poll::Ready(ret) => return Poll::Ready((task_id, ret)),
                        Poll::Pending => {}
                    }
                    task_id += 1;
                }
            }
            SelectFuture::OptFuture(ref mut tasks) => {
                for task_mut in tasks.iter_mut() {
                    if let Some(task) = task_mut {
                        let mut pin_fut =
                            unsafe { Pin::new_unchecked(std::ptr::read(task)) };
                        let task = pin_fut.as_mut().poll(cx);
                        std::mem::forget(pin_fut);
                        match task {
                            Poll::Ready(ret) => {
                                *task_mut = None;
                                return Poll::Ready((task_id, ret))
                            },
                            Poll::Pending => {}
                        }
                        task_id += 1;
                    }
                }
            }
        };
        Poll::Pending
    }
}

/// A trait to select on a slice of futures (or boxed futures).
///
/// # Select on slice of futures.
/// ```
/// use pasts::Select;
///
/// use core::future::Future;
/// use core::pin::Pin;
///
/// async fn async_main() {
///     let mut hello = async { "Hello" };
///     let mut world = async { "World!" };
///     // Hello is ready, so returns with index and result.
///     assert_eq!((0, "Hello"), [&mut hello as &mut dyn Future<Output=&str>, &mut world].select().await);
/// }
///
/// <pasts::ThreadInterrupt as pasts::Interrupt>::block_on(async_main());
/// ```
pub trait Select<'a, T> {
    /// Poll multiple futures, and return the future that's ready first.
    fn select(&mut self) -> SelectFuture<'a, '_, T>;
}

impl<'a, T> Select<'a, T> for [&'a mut dyn Future<Output = T>] {
    fn select(&mut self) -> SelectFuture<'a, '_, T> {
        SelectFuture::Future(self)
    }
}

impl<'a, T> Select<'a, T> for [Option<&'a mut dyn Future<Output = T>>] {
    fn select(&mut self) -> SelectFuture<'a, '_, T> {
        SelectFuture::OptFuture(self)
    }
}
