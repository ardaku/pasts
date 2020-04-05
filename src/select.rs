// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::{future::Future, pin::Pin, task::Poll, task::Context};

pub enum SelectFuture<'a, 'b, T> {
    Future(&'b mut [&'a mut Pin<&'a mut dyn Future<Output = T>>]),
    OptFuture(&'b mut [(&'a mut Pin<&'a mut dyn Future<Output = T>>, bool)]),
    #[cfg(feature = "std")]
    Boxed(&'b mut [&'a mut Pin<Box<dyn Future<Output = T>>>]),
    #[cfg(feature = "std")]
    OptBoxed(&'b mut [(&'a mut Pin<Box<dyn Future<Output = T>>>, bool)]),
}

impl<T> core::fmt::Debug for SelectFuture<'_, '_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Future(_) => write!(f, "Future"),
            Self::OptFuture(_) => write!(f, "OptFuture"),
            #[cfg(feature = "std")]
            Self::Boxed(_) => write!(f, "Boxed"),
            #[cfg(feature = "std")]
            Self::OptBoxed(_) => write!(f, "OptBoxed"),
        }
    }
}

impl<T> Future for SelectFuture<'_, '_, T> {
    type Output = (usize, T);
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut task_id = 0;
        match *self {
            SelectFuture::Future(ref mut tasks) => {
                for task in tasks.iter_mut().map(|a| a.as_mut().poll(cx)) {
                    match task {
                        Poll::Ready(ret) => return Poll::Ready((task_id, ret)),
                        Poll::Pending => {},
                    }
                    task_id += 1;
                }
            },
            SelectFuture::OptFuture(ref mut tasks) => {
                for task in tasks.iter_mut() {
                    if task.1 {
                        match task.0.as_mut().poll(cx) {
                            Poll::Ready(ret) => {
                                task.1 = false;
                                return Poll::Ready((task_id, ret))
                            },
                            Poll::Pending => {},
                        }
                    }
                    task_id += 1;
                }
            },
            #[cfg(feature = "std")]
            SelectFuture::Boxed(ref mut tasks) => {
                for task in tasks.iter_mut().map(|a| a.as_mut().poll(cx)) {
                    match task {
                        Poll::Ready(ret) => return Poll::Ready((task_id, ret)),
                        Poll::Pending => {},
                    }
                    task_id += 1;
                }
            },
            #[cfg(feature = "std")]
            SelectFuture::OptBoxed(ref mut tasks) => {
                for task in tasks.iter_mut() {
                    if task.1 {
                        match task.0.as_mut().poll(cx) {
                            Poll::Ready(ret) => {
                                task.1 = false;
                                return Poll::Ready((task_id, ret))
                            },
                            Poll::Pending => {},
                        }
                    }
                    task_id += 1;
                }
            },
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
///     pasts::pin_fut!(hello = async { "Hello" });
///     pasts::pin_fut!(world = async { "World!" });
///     // Hello is ready, so returns with index and result.
///     assert_eq!((0, "Hello"), [&mut hello, &mut world].select().await);
/// }
/// 
/// <pasts::ThreadInterrupt as pasts::Interrupt>::block_on(async_main());
/// ```
///
/// # Select on a slice of boxed futures.
/// ```
/// use pasts::Select;
///
/// use core::future::Future;
/// use core::pin::Pin;
///
/// async fn async_main() {
///     let mut hello: Pin<Box<dyn Future<Output=&str>>>
///         = Box::pin(async { "Hello" });
///     let mut world: Pin<Box<dyn Future<Output=&str>>>
///         = Box::pin(async { "World!" });
///     // Hello is ready, so returns with index and result.
///     assert_eq!((0, "Hello"), [&mut hello, &mut world].select().await);
/// }
/// 
/// <pasts::ThreadInterrupt as pasts::Interrupt>::block_on(async_main());
/// ```
pub trait Select<'a, T> {
    /// Poll multiple futures, and return the future that's ready first.
    fn select(&mut self) -> SelectFuture<'a, '_, T>;
}

impl<'a, T> Select<'a, T> for [&'a mut Pin<&'a mut dyn Future<Output = T>>] {
    fn select(&mut self) -> SelectFuture<'a, '_, T> {
        SelectFuture::Future(self)
    }
}

#[cfg(feature = "std")]
impl<'a, T> Select<'a, T> for [&'a mut Pin<Box<dyn Future<Output = T>>>] {
    fn select(&mut self) -> SelectFuture<'a, '_, T> {
        SelectFuture::Boxed(self)
    }
}

impl<'a, T> Select<'a, T> for [(&'a mut Pin<&'a mut dyn Future<Output = T>>, bool)] {
    fn select(&mut self) -> SelectFuture<'a, '_, T> {
        SelectFuture::OptFuture(self)
    }
}

#[cfg(feature = "std")]
impl<'a, T> Select<'a, T> for [(&'a mut Pin<Box<dyn Future<Output = T>>>, bool)] {
    fn select(&mut self) -> SelectFuture<'a, '_, T> {
        SelectFuture::OptBoxed(self)
    }
}
