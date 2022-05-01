// Copyright © 2019-2022 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use core::{fmt, future::Future, pin::Pin, task::Context};

use crate::{
    past::{Past, ToPast},
    prelude::*,
};

pub struct PollFn<F>(F);

impl<F> fmt::Debug for PollFn<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PollFn")
    }
}

impl<T, F: FnMut(&mut Context<'_>) -> Poll<T> + Unpin> Future for PollFn<F> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        self.0(cx)
    }
}

/// Polyfill for [`core::future::poll_fn`].
///
/// Create a [`Future`] from a repeating function returning [`Poll`].
///
/// ```rust
/// use pasts::prelude::*;
/// use core::task::Context;
///
/// fn read_line(_cx: &mut Context<'_>) -> Poll<String> {
///     Ready("Hello, World!".into())
/// }
///
/// pasts::block_on(async {
///     let read_future = pasts::poll_fn(read_line);
///     assert_eq!(read_future.await, "Hello, World!".to_owned());
/// });
/// ```
pub fn poll_fn<T, F>(f: F) -> PollFn<F>
where
    F: FnMut(&mut Context<'_>) -> Poll<T> + Unpin,
{
    PollFn(f)
}

pub struct PollNextFn<F>(F);

impl<F> fmt::Debug for PollNextFn<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PollNextFn")
    }
}

impl<T, F> ToPast<PollNextFn<F>, T> for PollNextFn<F>
where
    F: FnMut(&mut Context<'_>) -> Poll<T> + Unpin,
{
    fn to_past(self) -> Self {
        self
    }
}

impl<T, F> Past<T> for PollNextFn<F>
where
    F: FnMut(&mut Context<'_>) -> Poll<T> + Unpin,
{
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<T> {
        self.0(cx)
    }
}

/// Like [`poll_fn`] but for asynchronous iteration.
///
/// Useful for interoperability with other abstractions, such as `Stream` and
/// `AsyncIterator`.
///
/// ```rust
/// use pasts::{Loop, prelude::*};
///
/// struct State(u32);
///
/// impl State {
///     fn check(&mut self, value: Option<()>) -> Poll<u32> {
///         if value.is_none() {
///             return Ready(self.0);
///         }
///         self.0 += 1;
///         Pending
///     }
/// }
///
/// async fn run() {
///     let mut state = State(0);
///     let mut count = 3;
///     let past = pasts::poll_next_fn(|_cx| {
///         if count <= 0 {
///             return Ready(None);
///         }
///         count -= 1;
///         Ready(Some(()))
///     });
///
///     let count = Loop::new(&mut state)
///         .on(past, State::check)
///         .await;
///
///     assert_eq!(count, 3);
/// }
///
/// pasts::block_on(run());
/// ```
pub fn poll_next_fn<T, F>(f: F) -> PollNextFn<F>
where
    F: FnMut(&mut Context<'_>) -> Poll<T> + Unpin,
{
    PollNextFn(f)
}
