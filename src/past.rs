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
pub struct Past<O, F = Box<dyn FnMut(&mut Context<'_>) -> Poll<O> + Send>>
where
    F: FnMut(&mut Context<'_>) -> Poll<O>,
{
    poll_next: F,
}

impl<O> Past<O> {
    /// Creates a [`Past`] that wraps a function returning a `!`[`Unpin`] future.
    ///
    /// Note that this API does require an allocator.
    #[inline(always)]
    pub fn pin<T, N>(mut future_fn: N) -> Self
    where
        T: Future<Output = O> + Send + 'static,
        N: FnMut() -> T + Send + 'static,
    {
        let mut boxy = Box::pin((future_fn)());

        Past::with(Box::new(move |cx| {
            boxy.as_mut().poll(cx).map(|output| {
                boxy.set((future_fn)());
                output
            })
        }))
    }

    /// Creates a [`Past`] from an infinite iterator of futures.
    #[inline(always)]
    pub fn new<T, R>(
        iter: impl IntoIterator<Item = T, IntoIter = RepeatWith<R>>,
    ) -> Past<O, impl FnMut(&mut Context<'_>) -> Poll<O>>
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

impl<O, F: FnMut(&mut Context<'_>) -> Poll<O> + Send> Past<O, F> {
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

pub trait Pasty<O>: Unpin {
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<O>;
}

pub trait ToPast<P: Pasty<O>, O> {
    fn to_past(self) -> P;
}

impl<O, F, T> ToPast<T, (usize, O)> for T
where
    F: FnMut(&mut Context<'_>) -> Poll<O> + Unpin,
    T: core::ops::DerefMut<Target = [Past<O, F>]> + Unpin,
{
    fn to_past(self) -> Self {
        self
    }
}

impl<O, F, T> Pasty<(usize, O)> for T
where
    F: FnMut(&mut Context<'_>) -> Poll<O> + Unpin,
    T: core::ops::DerefMut<Target = [Past<O, F>]> + Unpin,
{
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<(usize, O)> {
        for (i, this) in self.iter_mut().enumerate() {
            if let Poll::Ready(value) = (this.poll_next)(cx) {
                return Poll::Ready((i, value));
            }
        }
        Poll::Pending
    }
}

impl<O, F> ToPast<Past<O, F>, O> for Past<O, F>
where
    F: FnMut(&mut Context<'_>) -> Poll<O> + Unpin,
{
    fn to_past(self) -> Self {
        self
    }
}

impl<O, F> Pasty<O> for Past<O, F>
where
    F: FnMut(&mut Context<'_>) -> Poll<O> + Unpin,
{
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<O> {
        (self.poll_next)(cx)
    }
}

#[derive(Debug)]
pub struct PastIter<O, F, N>
where
    F: Future<Output = O> + Unpin + Send,
    N: FnMut() -> F,
{
    future: F,
    iter: RepeatWith<N>,
}

impl<O, F, N> Pasty<O> for PastIter<O, F, N>
where
    F: Future<Output = O> + Unpin + Send,
    N: FnMut() -> F + Unpin,
{
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<O> {
        Pin::new(&mut self.future).poll(cx).map(|output| {
            self.future = self.iter.next().unwrap_or_else(|| unreachable!());
            output
        })
    }
}

impl<O, F, N, T> ToPast<PastIter<O, F, N>, O> for T
where
    F: Future<Output = O> + Unpin + Send,
    T: IntoIterator<Item = F, IntoIter = RepeatWith<N>>,
    N: FnMut() -> F + Unpin,
{
    fn to_past(self) -> PastIter<O, F, N> {
        let mut iter = self.into_iter();
        let future = iter.next().unwrap_or_else(|| unreachable!());

        PastIter { iter, future }
    }
}

struct Fut<'a, O, F>(&'a mut Past<O, F>)
where
    F: FnMut(&mut Context<'_>) -> Poll<O> + Send;

impl<F: FnMut(&mut Context<'_>) -> Poll<O> + Send, O> Future for Fut<'_, O, F> {
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

#[derive(Debug)]
pub struct Never<'a, S>(&'a mut S);

impl<S, T> Stateful<S, T> for Never<'_, S> {
    fn state(&mut self) -> &mut S {
        self.0
    }

    fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<Poll<T>> {
        Poll::Pending
    }
}

/// Composable asynchronous event loop.
#[derive(Debug)]
pub struct Loop<S: Unpin, T, F: Stateful<S, T>> {
    other: F,
    _phantom: core::marker::PhantomData<(S, T)>,
}

impl<'a, S: Unpin, T> Loop<S, T, Never<'a, S>> {
    /// Create an empty event loop.
    pub fn new(state: &'a mut S) -> Self {
        let other = Never(state);
        let _phantom = core::marker::PhantomData;

        Loop { other, _phantom }
    }
}

impl<S: Unpin, T, F: Stateful<S, T>> Loop<S, T, F> {
    /// Register an event handler.
    ///
    /// Parameter `past` may be one of:
    ///  - [`Past`]:  
    ///    `output` passed to handler
    ///  - [`DerefMut`](core::ops::DerefMut)`<`[`[`](slice)[`Past`](crate::Past)[`]`](slice)`>`:  
    ///    `(index, output)` passed to handler
    ///  - [`IntoIterator`]`<IntoIter = `[`RepeatWith`](core::iter::RepeatWith)`<`[`Future`](core::future::Future)`>>`:  
    ///    `output` passed to handler
    pub fn on<P, O, N>(
        self,
        past: P,
        then: fn(&mut S, O) -> Poll<T>,
    ) -> Loop<S, T, impl Stateful<S, T>>
    where
        P: ToPast<N, O>,
        N: Pasty<O>,
    {
        let past = past.to_past();
        let other = self.other;
        let _phantom = core::marker::PhantomData;
        let other = Join { other, past, then };

        Loop { other, _phantom }
    }
}

impl<S: Unpin, T: Unpin, F: Stateful<S, T>> Future for Loop<S, T, F> {
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

struct Join<S, T, O, F: Stateful<S, T>, P: Pasty<O>> {
    other: F,
    past: P,
    then: fn(&mut S, O) -> Poll<T>,
}

impl<S, T, O, F, P> Stateful<S, T> for Join<S, T, O, F, P>
where
    F: Stateful<S, T>,
    P: Pasty<O>,
{
    fn state(&mut self) -> &mut S {
        self.other.state()
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Poll<T>> {
        if let Poll::Ready(output) = self
            .past
            .poll_next(cx)
            .map(|output| (self.then)(self.other.state(), output))
        {
            Poll::Ready(output)
        } else {
            self.other.poll(cx)
        }
    }
}
