// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::{
    future::Future, marker::PhantomData, pin::Pin, task::Context, task::Poll,
};

#[allow(missing_debug_implementations)]
pub struct SelectFuture<'b, T: Unpin + 'b, F: Future<Output = T> + Unpin>(
    &'b mut [F],
    PhantomData<T>,
);

impl<T: Unpin, F: Future<Output = T> + Unpin> Future
    for SelectFuture<'_, T, F>
{
    type Output = (usize, T);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let tasks = &mut self.get_mut().0;

        for (task_id, mut task) in tasks.iter_mut().enumerate() {
            let pin_fut = Pin::new(&mut task);
            let task = pin_fut.poll(cx);
            match task {
                Poll::Ready(ret) => return Poll::Ready((task_id, ret)),
                Poll::Pending => {}
            }
        }

        Poll::Pending
    }
}

/// A trait to select on a slice of `Future`s.
///
/// # Select on slice of futures.
///
/// ```rust
/// use pasts::prelude::*;
///
/// async fn async_main() {
///     task!(let hello = async { "Hello" });
///     task!(let world = async { "World!"});
///     // Hello is ready, so returns with index and result.
///     assert_eq!((0, "Hello"), [hello, world].select().await);
/// }
///
/// pasts::spawn(async_main);
/// ```
// Future needs to be unpin to prevent UB because `Future`s can move between
// calls to select after starting (which fills future's RAM with garbage data).
pub trait Select<'a, T: Unpin, F: Future<Output = T> + Unpin> {
    /// Poll multiple futures, and return the value from the future that returns
    /// `Ready` first.
    fn select(&'a mut self) -> SelectFuture<'a, T, F>;
}

impl<'a, T: Unpin, F: Future<Output = T> + Unpin> Select<'a, T, F> for [F] {
    fn select(&'a mut self) -> SelectFuture<'a, T, F> {
        SelectFuture(self, PhantomData)
    }
}
