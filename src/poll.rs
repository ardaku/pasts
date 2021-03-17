// Pasts
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use core::task::{Poll as Pending, Context};
use core::pin::Pin;
use core::future::Future;

#[derive(Debug)]
pub struct PollFuture<'a, T, F: Future<Output = T> + Unpin>(&'a mut [F]);

pub trait Sealed<T> {}

/// A trait that turns a slice, vec or array of futures into a future.
///
/// # Example: Await on The Fastest Future
/// This is the pasts way of doing the futures crate's `select!()`.  Note
/// however that works completely differently.  Also, if you're writing an
/// asynchronous loop, use [`Loop`](crate::Loop) instead.
///
/// ```
/// use pasts::{Task, Poll};
///
/// pasts::glue!();
///
/// async fn run() {
///     let hello: Task<&str> = Box::pin(async { "Hello" });
///     let world: Task<&str> = Box::pin(async { "World" });
///     let mut array = [hello, world];
///     // Hello is ready, so returns with index and result.
///     assert_eq!((0, "Hello"), array.poll().await);
/// }
/// ```
///
/// # Another Example: Dynamic Join
/// This is the pasts way of doing the futures crate's `join!()`, given that you
/// can accept a bit of allocation.  This is not usually the case, and is more
/// useful for concurrently executing a dynamic number of tasks.
///
/// ```
/// use pasts::{Task, Poll};
/// 
/// pasts::glue!();
/// 
/// async fn run() {
///     let hello: Task<&str> = Box::pin(async { "Hello" });
///     let world: Task<&str> = Box::pin(async { "World" });
///     let mut tasks = vec![hello, world];
/// 
///     while !tasks.is_empty() {
///         let (idx, val) = tasks.poll().await;
///         tasks.remove(idx);
///         println!("Received message from completed task: {}", val);
///     }
/// }
/// ```
pub trait Poll<T, F: Future<Output = T> + Unpin>: Sealed<T> + Unpin {
    /// Create a future that polls all contained futures in the slice.
    fn poll(&mut self) -> PollFuture<'_, T, F>;
}

impl<T, F: Future<Output = T> + Unpin> Poll<T, F> for [F] {
    fn poll(&mut self) -> PollFuture<'_, T, F> {
        PollFuture(self)
    }
}

impl<T, F: Future<Output = T> + Unpin> Sealed<T> for [F] { }

impl<T, F: Future<Output = T> + Unpin> Future for PollFuture<'_, T, F> {
    type Output = (usize, T);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Pending<(usize, T)> {
        let this = self.get_mut();
        for (task_id, mut task) in this.0.iter_mut().enumerate() {
            let pin_fut = Pin::new(&mut task);
            let task = pin_fut.poll(cx);
            match task {
                Pending::Ready(ret) => return Pending::Ready((task_id, ret)),
                Pending::Pending => {}
            }
        }
        Pending::Pending
    }
}
