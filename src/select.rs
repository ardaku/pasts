// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::{fmt::Debug, future::Future, pin::Pin, task::Context, task::Poll};

pub enum SelectFuture<'b, T, A: Future<Output = T> + Unpin> {
    Future(&'b mut [A]),
    OptFuture(&'b mut [Option<A>]),
}

impl<T, A: Future<Output = T> + Unpin> Debug for SelectFuture<'_, T, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Future(_) => write!(f, "Future"),
            Self::OptFuture(_) => write!(f, "OptFuture"),
        }
    }
}

impl<T, A: Future<Output = T> + Unpin> Future for SelectFuture<'_, T, A> {
    type Output = (usize, T);

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        match *self {
            SelectFuture::Future(ref mut tasks) => {
                for (task_id, task) in tasks.iter_mut().enumerate() {
                    let pin_fut = Pin::new(task);
                    let task = pin_fut.poll(cx);
                    match task {
                        Poll::Ready(ret) => return Poll::Ready((task_id, ret)),
                        Poll::Pending => {}
                    }
                }
            }
            SelectFuture::OptFuture(ref mut tasks) => {
                for (task_id, task_opt) in tasks.iter_mut().enumerate() {
                    if let Some(ref mut task) = task_opt {
                        let pin_fut = Pin::new(task);
                        let task = pin_fut.poll(cx);
                        match task {
                            Poll::Ready(ret) => {
                                *task_opt = None;
                                return Poll::Ready((task_id, ret));
                            }
                            Poll::Pending => {}
                        }
                    }
                }
            }
        };
        Poll::Pending
    }
}

/// A trait to select on a slice of `Future`s or `Option<Future>`s.
///
/// # Select on slice of futures.
#[cfg_attr(
    feature = "std",
    doc = r#"
```rust
use pasts::prelude::*;
use pasts::CvarExec;

static EXECUTOR: CvarExec = CvarExec::new();

async fn async_main() {
    let mut hello = async { "Hello" };
    let mut world = async { "World!" };
    // Hello is ready, so returns with index and result.
    assert_eq!((0, "Hello"), [hello.fut(), world.fut()].select().await);
}

EXECUTOR.block_on(async_main());
```
"#
)]
// Future needs to be unpin to prevent UB because `Future`s can move between
// calls to select after starting (which fills future's RAM with garbage data).
pub trait Select<T, A: Future<Output = T> + Unpin> {
    /// Poll multiple futures, and return the value from the future that returns
    /// `Ready` first.
    fn select(&mut self) -> SelectFuture<'_, T, A>;
}

impl<T, A: Future<Output = T> + Unpin> Select<T, A> for [A] {
    fn select(&mut self) -> SelectFuture<'_, T, A> {
        SelectFuture::Future(self)
    }
}

impl<T, A: Future<Output = T> + Unpin> Select<T, A> for [Option<A>] {
    fn select(&mut self) -> SelectFuture<'_, T, A> {
        SelectFuture::OptFuture(self)
    }
}
