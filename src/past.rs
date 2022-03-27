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
    iter::RepeatWith,
};

/// Infinite asynchronous iterator.
/// 
/// You can create a `Past` with one of three functions:
///  - [`Past::pin`] Using a closure to create a future, usable with `async fn`s
///  - [`Past::new`] Create a past from a future iterator (recommended)
///  - [`Past::with`] Create a past from a function
///    
#[derive(Debug)]
pub struct Past<F: FnMut(&mut Context<'_>) -> Poll<O>, O = ()> {
    poll_next: F,
}

impl<O> Past<fn(&mut Context<'_>) -> Poll<O>, O> {
    /// Creates a [`Past`] that wraps a function returning a `!`[`Unpin`] future.
    ///
    /// Note that this API does require an allocator.
    #[inline(always)]
    pub fn pin<T, N>(mut future_fn: N) -> Past<impl FnMut(&mut Context<'_>) -> Poll<O>, O>
        where T: Future<Output = O> + Send + 'static, N: FnMut() -> T + Send + 'static
    {
        let mut boxy = Box::pin((future_fn)());
        Past::with(move |cx| {
            boxy.as_mut().poll(cx).map(|output| {
                boxy.set((future_fn)());
                output
            })
        })
    }

    /// Creates a [`Past`] from an infinite iterator of futures.
    pub fn new<I, T, R>(iter: I) -> Past<impl FnMut(&mut Context<'_>) -> Poll<O>, O>
        where I: IntoIterator<Item = T, IntoIter = RepeatWith<R>>,
              T: Future<Output = O> + Unpin + Send,
              R: FnMut() -> T + Send,
    {
        let mut iter = iter.into_iter();
        let mut fut = iter.next().unwrap_or_else(|| unreachable!());
        Past::with(move |cx| {
            Pin::new(&mut fut).poll(cx).map(|output| {
                fut = iter.next().unwrap_or_else(|| unreachable!());
                output
            })
        })
    }
}

impl<F, O> Past<F, O>
    where F: FnMut(&mut Context<'_>) -> Poll<O> + Send,
{

    /// Creates a [`Past`] that wraps a function returning
    /// [`Poll`](core::task::Poll).
    #[inline(always)]
    pub fn with(poll_next: F) -> Self {
        Past {
            poll_next,
        }
    }

    /// Get a new [`Unpin`] + [`Send`] future ready on next I/O completion.
    // Because this is an "async iterator", and doesn't ever return `None`.
    #[allow(clippy::should_implement_trait)]
    #[inline(always)]
    pub fn next(&mut self) -> impl Future<Output = O> + Send + Unpin + '_ {
        Fut(self)
    }
}

struct Fut<'a, F: FnMut(&mut Context<'_>) -> Poll<O> + Send, O>(&'a mut Past<F, O>);

impl<F: FnMut(&mut Context<'_>) -> Poll<O> + Send, O> Future for Fut<'_, F, O> {
    type Output = O;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<O> {
        (self.0.poll_next)(cx)
    }
}
