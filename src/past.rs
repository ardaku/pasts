// Copyright © 2019-2022 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

type Fut<T> = Pin<Box<T>>;

/// A repeating `async fn`.
///
/// This is needed to both pin an `async fn`, and avoid panics of `async fn`
/// being polled after it returns `Poll::Ready()`.  Note that this does
/// allocate upon creation.
#[allow(missing_debug_implementations)]
pub struct Past<S: Unpin, T, F: Future<Output = T>>(S, fn(&mut S) -> F, Fut<F>);

impl<S: Unpin, T, F: Future<Output = T>> Past<S, T, F> {
    /// Create a new repeating `Unpin` async function.
    pub fn new(mut state: S, async_fn: fn(&mut S) -> F) -> Self {
        let future = Box::pin((async_fn)(&mut state));
        Self(state, async_fn, future)
    }
}

impl<S: Unpin, T, F: Future<Output = T>> Future for Past<S, T, F> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let poll = self.2.as_mut().poll(cx);
        if poll.is_ready() {
            let new = self.1(&mut self.0);
            self.2.set(new);
        }
        poll
    }
}
