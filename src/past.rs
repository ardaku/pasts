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

/// Type-erased repeating async function
#[allow(missing_debug_implementations)]
pub struct Task<O>(Box<dyn Pasty<O>>);

impl<O> Task<O> {
    /// Create a new type-erased task.
    pub fn new<P: Pasty<O> + 'static, N: ToPast<P, O>>(func: N) -> Self {
        Task(Box::new(func.to_past()))
    }

    /// Poll for next output.
    pub fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<O> {
        self.0.as_mut().poll_next(cx)
    }
}

pub trait Pasty<O>: Unpin {
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<O>;
}

impl<O> Pasty<O> for Task<O> {
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<O> {
        self.0.as_mut().poll_next(cx)
    }
}

pub trait ToPast<P: Pasty<O>, O> {
    fn to_past(self) -> P;
}

struct FnWrapper<O, T: FnMut(&mut Context<'_>) -> Poll<O> + Unpin>(T);

impl<O, T> ToPast<FnWrapper<O, T>, O> for T
where
    T: FnMut(&mut Context<'_>) -> Poll<O> + Unpin,
{
    fn to_past(self) -> FnWrapper<O, Self> {
        FnWrapper(self)
    }
}

impl<O, T> Pasty<O> for FnWrapper<O, T>
where
    T: FnMut(&mut Context<'_>) -> Poll<O> + Unpin,
{
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<O> {
        (self.0)(cx)
    }
}

impl<O, T, D> ToPast<T, (usize, O)> for T
where
    T: core::ops::DerefMut<Target = [D]> + Unpin,
    D: Pasty<O>,
{
    fn to_past(self) -> Self {
        self
    }
}

impl<O, T, D> Pasty<(usize, O)> for T
where
    T: core::ops::DerefMut<Target = [D]> + Unpin,
    D: Pasty<O>,
{
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<(usize, O)> {
        for (i, this) in self.iter_mut().enumerate() {
            if let Poll::Ready(value) = this.poll_next(cx) {
                return Poll::Ready((i, value));
            }
        }
        Poll::Pending
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

#[derive(Debug)]
pub struct BoxedPastIter<O, F, N>
where
    F: Future<Output = O> + Send,
    N: (FnMut() -> F) + Unpin,
{
    future: Pin<Box<F>>,
    next: N,
}

impl<O, F, N> Pasty<O> for BoxedPastIter<O, F, N>
where
    F: Future<Output = O> + Send,
    N: (FnMut() -> F) + Unpin,
{
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<O> {
        Pin::new(&mut self.future).poll(cx).map(|output| {
            self.future.set((self.next)());
            output
        })
    }
}

impl<O, F, N> ToPast<BoxedPastIter<O, F, N>, O> for N
where
    F: Future<Output = O> + Send,
    N: (FnMut() -> F) + Unpin,
{
    fn to_past(mut self) -> BoxedPastIter<O, F, N> {
        let future = Box::pin((self)());
        let next = self;

        BoxedPastIter { next, future }
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
    ///  - [`IntoIterator`]`<IntoIter = `[`RepeatWith`](core::iter::RepeatWith)`<`[`Future`](core::future::Future)`>>`:  
    ///    (future must be [`Unpin`])
    ///  - An async function (no parameters) / closure that returns a future (allocates):  
    ///  - A poll function (`FnMut(&mut Context<'_>) -> Poll<O>`)
    ///  - Anything that dereferences to a slice of any of the above
    ///    (passes `(usize, O)` to `then()`)
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
