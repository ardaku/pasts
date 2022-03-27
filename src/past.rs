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

/// A boxed, pinned future.
///
/// You should use this in conjunction with
/// [`Loop::poll`](crate::Loop::poll) if you need a dynamic number of tasks (for
/// instance, a web server).
///
/// # Example
/// This example spawns two tasks on the same thread, and then terminates when
/// they both complete.
///
/// ```
/// use pasts::{prelude::*, Loop, Task};
///
/// enum Exit {
///     /// Task has completed, remove it
///     Remove(usize),
/// }
///
/// struct State {}
///
/// impl State {
///     fn completion(&mut self, (id, val): (usize, &str)) -> Poll<Exit> {
///         println!("Received message from completed task: {}", val);
///
///         Ready(Exit::Remove(id))
///     }
/// }
///
/// async fn run() {
///     let mut state = State {};
///     let mut tasks = vec![
///         Task::pin(|| async { "Hello" }),
///         Task::pin(|| async { "World" }),
///     ];
///
///     while !tasks.is_empty() {
///         match Loop::new(&mut state)
///             .on(tasks.as_mut_slice(), State::completion)
///             .await
///         {
///             Exit::Remove(index) => {
///                 tasks.swap_remove(index);
///             }
///         }
///     }
/// }
///
/// fn main() {
///     pasts::block_on(run());
/// }
/// ```
pub type Task<O> = Past<Box<dyn FnMut(&mut Context<'_>) -> Poll<O> + Send>, O>;

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

impl<O> Past<Box<dyn FnMut(&mut Context<'_>) -> Poll<O> + Send>, O> {
    /// Creates a [`Past`] that wraps a function returning a `!`[`Unpin`] future.
    ///
    /// Note that this API does require an allocator.
    #[inline(always)]
    pub fn pin<T, N>(mut future_fn: N) -> Task<O>
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

impl<F: FnMut(&mut Context<'_>) -> Poll<O> + Send, O> Past<F, O> {
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

impl<F, O, T> ToPast<T, (usize, O)> for T
where
    F: FnMut(&mut Context<'_>) -> Poll<O> + Unpin,
    T: core::ops::DerefMut<Target = [Past<F, O>]> + Unpin,
{
    fn to_past(self) -> Self {
        self
    }
}

impl<F, O, T> Pasty<(usize, O)> for T
where
    F: FnMut(&mut Context<'_>) -> Poll<O> + Unpin,
    T: core::ops::DerefMut<Target = [Past<F, O>]> + Unpin,
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

impl<F, O> ToPast<Past<F, O>, O> for Past<F, O>
where
    F: FnMut(&mut Context<'_>) -> Poll<O> + Unpin,
{
    fn to_past(self) -> Self {
        self
    }
}

impl<F, O> Pasty<O> for Past<F, O>
where
    F: FnMut(&mut Context<'_>) -> Poll<O> + Unpin,
{
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<O> {
        (self.poll_next)(cx)
    }
}

#[derive(Debug)]
pub struct PastIter<F, O, N>
where
    F: Future<Output = O> + Unpin + Send,
    N: FnMut() -> F,
{
    future: F,
    iter: RepeatWith<N>,
}

impl<F, O, N> Pasty<O> for PastIter<F, O, N>
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

impl<F, O, N, T> ToPast<PastIter<F, O, N>, O> for T
where
    F: Future<Output = O> + Unpin + Send,
    T: IntoIterator<Item = F, IntoIter = RepeatWith<N>>,
    N: FnMut() -> F + Unpin,
{
    fn to_past(self) -> PastIter<F, O, N> {
        let mut iter = self.into_iter();
        let future = iter.next().unwrap_or_else(|| unreachable!());

        PastIter { iter, future }
    }
}

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
    /// Register a callback.
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

struct Join<S, T, F: Stateful<S, T>, O, P: Pasty<O>> {
    other: F,
    past: P,
    then: fn(&mut S, O) -> Poll<T>,
}

impl<S, T, F, O, P> Stateful<S, T> for Join<S, T, F, O, P>
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
