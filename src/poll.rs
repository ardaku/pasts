// Pasts
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

/// Create a future that waits on multiple futures concurrently and returns the
/// first result.
///
/// Takes an array of types that implement [`Future`](core::future::Future) and
/// [`Unpin`](core::marker::Unpin).
///
/// # Example: Await on The Fastest Future
/// `race!()` will always poll the first future in the array first.
///
/// ```rust
/// use core::{future::Future, pin::Pin};
/// use pasts::race;
///
/// async fn async_main() {
///     let hello: Pin<Box<dyn Future<Output=&str>>> = Box::pin(async { "Hello" });
///     let world: Pin<Box<dyn Future<Output=&str>>> = Box::pin(async { "World" });
///     let mut array = [hello, world];
///     // Hello is ready, so returns with index and result.
///     assert_eq!((0, "Hello"), race!(array));
/// }
///
/// pasts::block_on(async_main());
/// ```
#[macro_export]
macro_rules! race {
    ($f:expr) => {{
        use core::{
            future::Future,
            pin::Pin,
            task::{Context, Poll},
        };
        struct Fut<'a, T, F: Future<Output = T> + Unpin> {
            futures: &'a mut [F],
        }
        impl<T, F: Future<Output = T> + Unpin> Unpin for Fut<'_, T, F> {}
        impl<T: Unpin, F: Future<Output = T> + Unpin> Future for Fut<'_, T, F> {
            type Output = (usize, T);
            fn poll(
                self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<Self::Output> {
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
        Pin::<&mut (dyn Future<Output = _> + Unpin)>::new(&mut Fut {
            futures: &mut $f[..],
        })
        .await
    }};
}

/// Similar to [`race!()`], except doesn't take an array, but rather a list of
/// asynchronous expressions.
#[macro_export]
macro_rules! wait {
    ($($f:expr),* $(,)?) => {{
        use core::{pin::Pin, future::Future};

        // Safe because future can't move because it can't be directly
        // accessed
        $crate::race!([$(unsafe {
            Pin::<&mut dyn Future<Output = _>>::new_unchecked(
                &mut async { $f }
            )
        }),*]).1
    }};
}
