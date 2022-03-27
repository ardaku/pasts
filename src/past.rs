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
    iter::RepeatWith,
    pin::Pin,
    task::{Context, Poll},
};

/// Infinite asynchronous iterator.
///
/// You can create a `Past` with one of three functions:
///  - [`Past::pin`] Using a closure to create a future, usable with `async fn`s
///  - [`Past::new`] Create a past from a future iterator (recommended)
///  - [`Past::with`] Create a past from a function
#[derive(Debug)]
pub struct Past<F: FnMut(&mut Context<'_>) -> Poll<O>, O = ()> {
    poll_next: F,
}

impl<O> Past<fn(&mut Context<'_>) -> Poll<O>, O> {
    /// Creates a [`Past`] that wraps a function returning a `!`[`Unpin`] future.
    ///
    /// Note that this API does require an allocator.
    #[inline(always)]
    pub fn pin<T, N>(
        mut future_fn: N,
    ) -> Past<impl FnMut(&mut Context<'_>) -> Poll<O>, O>
    where
        T: Future<Output = O> + Send + 'static,
        N: FnMut() -> T + Send + 'static,
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
    #[inline(always)]
    pub fn new<T, R>(
        iter: impl IntoIterator<Item = T, IntoIter = RepeatWith<R>>,
    ) -> Past<impl FnMut(&mut Context<'_>) -> Poll<O>, O>
    where
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
where
    F: FnMut(&mut Context<'_>) -> Poll<O> + Send,
{
    /// Creates a [`Past`] that wraps a function returning
    /// [`Poll`](core::task::Poll).
    #[inline(always)]
    pub fn with(poll_next: F) -> Self {
        Past { poll_next }
    }

    /// Get a new [`Unpin`] + [`Send`] future ready on next I/O completion.
    // Because this is an "async iterator", and doesn't ever return `None`.
    #[allow(clippy::should_implement_trait)]
    #[inline(always)]
    pub fn next(&mut self) -> impl Future<Output = O> + Send + Unpin + '_ {
        Fut(self)
    }
}

/*impl<F, O, T> From<T> for Past<F, O>
    where T: IntoIterator<Item = T, IntoIter = RepeatWith<R>>
{

}*/

struct Fut<'a, F, O>(&'a mut Past<F, O>)
where
    F: FnMut(&mut Context<'_>) -> Poll<O> + Send;

impl<F: FnMut(&mut Context<'_>) -> Poll<O> + Send, O> Future for Fut<'_, F, O> {
    type Output = O;

    #[inline(always)]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<O> {
        (self.0.poll_next)(cx)
    }
}

pub trait Stateful<S, T>: Unpin {
    fn state(&mut self) -> &mut S;

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Poll<T>>;
}

struct Never<'a, S>(&'a mut S);

impl<S, T> Stateful<S, T> for Never<'_, S> {
    fn state(&mut self) -> &mut S {
        self.0
    }

    fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<Poll<T>> {
        Poll::Pending
    }
}

/// Composable event loop.
#[derive(Debug)]
pub struct Loop2<S: Unpin, T, F: Stateful<S, T>> {
    other: F,
    _phantom: core::marker::PhantomData<(S, T)>,
}

impl<'a, S: Unpin, T> Loop2<S, T, Never<'a, S>> {
    /// Create an empty event loop.
    pub fn new(state: &'a mut S) -> Self {
        let other = Never(state);
        let _phantom = core::marker::PhantomData;

        Loop2 {
            other, _phantom
        }
    }
}

impl<S: Unpin, T, F: Stateful<S, T>> Loop2<S, T, F> {
    /// Register a callback.
    pub fn on<P, O, N>(self, past: P, then: fn(&mut S, O) -> Poll<T>) -> Loop2<S, T, impl Stateful<S, T>>
        where P: Into<Past<N, O>>, N: FnMut(&mut Context<'_>) -> Poll<O> + Unpin
    {
        let past = past.into();
        let other = self.other;
        let _phantom = core::marker::PhantomData;
        let other = Join {
            other,
            past,
            then,
        };

        Loop2 {
            other, _phantom
        }
    }
}

impl<S: Unpin, T: Unpin, F: Stateful<S, T>> Future for Loop2<S, T, F> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let this = self.get_mut();
        while let Poll::Ready(output) = Pin::new(&mut this.other).poll(cx) {
            if let Poll::Ready(output) = output {
                return Poll::Ready(output);
            }
        }

        Poll::Pending
    }
}

struct Join<S, T, F: Stateful<S, T>, N, O>
where
    N: FnMut(&mut Context<'_>) -> Poll<O> + Unpin
{
    other: F,
    past: Past<N, O>,
    then: fn(&mut S, O) -> Poll<T>,
}

impl<S, T, F: Stateful<S, T>, N, O> Stateful<S, T> for Join<S, T, F, N, O>
where
    N: FnMut(&mut Context<'_>) -> Poll<O> + Unpin
{
    fn state(&mut self) -> &mut S {
        self.other.state()
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Poll<T>> {
        if let Poll::Ready(output) = (self.past.poll_next)(cx).map(|output| {
            (self.then)(self.other.state(), output)
        }) {
            Poll::Ready(output)
        } else {
            self.other.poll(cx)
        }
    }
}
