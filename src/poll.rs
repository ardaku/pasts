// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

/// Poll multiple futures concurrently.
///
/// Takes an array of types that implement [`Future`](core::future::Future) and
/// [`Unpin`](core::marker::Unpin).  If you're dealing with futures that don't
/// implement [`Unpin`](core::marker::Unpin), you can use the
/// [`task!()`](crate::task) macro to make it implement
/// [`Unpin`](core::marker::Unpin).  The resulting type will be the same as
/// other futures created with [`task!()`](crate::task).
///
/// # Examples
/// ## Await on The Fastest Future
/// `poll!()` will always poll the first future in the array first.
///
/// ```rust
/// use pasts::prelude::*;
///
/// async fn async_main() {
///     task!(let hello = async { "Hello" });
///     task!(let world = async { "World!"});
///     // Hello is ready, so returns with index and result.
///     assert_eq!((0, "Hello"), poll!(hello, world).await);
/// }
///
/// pasts::spawn(async_main);
/// ```
///
/// ## Await the Outputs of Two Futures Concurrently
/// ```rust
/// use pasts::prelude::*;
///
/// async fn async_main() {
///     task!(let hello = async { "Hello" });
///     task!(let world = async { "World!"});
///     let mut task_queue = [hello, world];
///     let mut results = (None, None);
///     for _ in 0u8..2 {
///         match poll!(task_queue).await {
///             (0, a) => results.0 = Some(a),
///             (1, a) => results.1 = Some(a),
///             _ => unreachable!(),
///         }
///     }
///     let results = (results.0.unwrap(), results.1.unwrap());
///     assert_eq!(("Hello", "World!"), results);
/// }
/// ```
#[macro_export]
macro_rules! poll {
    ($f:expr) => {{
        use core::{future::Future, task::{Context, Poll}, pin::Pin};
        struct Fut<'a, T, F: Future<Output = T> + Unpin> {
            futures: &'a mut [F],
        }
        impl<T: Unpin, F: Future<Output = T> + Unpin> Future for Fut<'_, T, F> {
            type Output = (usize, T);
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>)
                -> Poll<Self::Output>
            {
                let tasks = &mut self.get_mut().futures;
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
        Pin::<&mut (dyn Future<Output=_> + Unpin)>::new(
            &mut Fut { futures: &mut $f[..] }
        )
    }};

    ($($f:expr),* $(,)?) => {{
        poll!([$(&mut $f),*])
    }};
}
