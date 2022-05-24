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
    cell::{RefCell, RefMut},
    future::Future,
    iter::Fuse,
    pin::Pin,
    task::{Context, Poll},
};

use crate::prelude::*;

/// Trait for asynchronous iteration.
///
/// Polyfill for [`AsyncIterator`](core::async_iter::AsyncIterator), with some
/// extra provided "async" methods (they return futures).
pub trait AsyncIterator {
    /// The type that is yielded by this async iterator
    type Item;

    /// Attempt to get the next value from this iterator, registering a wakeup
    /// when not ready.
    ///
    /// Returns `None` when the iterator is exhausted.
    ///
    /// # Return Value
    ///  - `Poll::Pending` - Not ready yet
    ///  - `Poll::Ready(Some(val))` - Ready with value
    ///  - `Poll::Ready(None)` - Ready with close, `poll_next()` shouldn't be
    ///    called again (may panic).
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>>;

    /// Get the next `Option<Self::Item>`
    ///
    /// ```rust
    /// use pasts::{AsyncIterator, prelude::*};
    ///
    /// struct MyAsyncIter;
    ///
    /// impl AsyncIterator for MyAsyncIter {
    ///     type Item = u32;
    ///
    ///     fn poll_next(
    ///         self: Pin<&mut Self>,
    ///         _cx: &mut Context<'_>
    ///     ) -> Poll<Option<Self::Item>> {
    ///         Ready(Some(1))
    ///     }
    /// }
    ///
    /// async fn run() {
    ///     let mut count = 0;
    ///     let mut async_iter = MyAsyncIter;
    ///     let mut iterations = 0;
    ///     while let Some(i) = async_iter.next().await {
    ///         count += i;
    ///         iterations += 1;
    ///         if iterations == 3 {
    ///             break;
    ///         }
    ///     }
    ///     assert_eq!(count, 3);
    /// }
    ///
    /// pasts::block_on(run());
    /// ```
    fn next(&mut self) -> IterNextFuture<'_, Self::Item>
    where
        Self: Sized + Unpin,
    {
        IterNextFuture(self)
    }

    /// Convert this `AsyncIterator` into an [`AsyncIter`].
    ///
    /// This function returns a [`Future`] that fetches the first item, if you
    /// want an unprepared [`Iterator`], use [`AsyncIter::new()`]
    ///
    /// ```rust
    /// use pasts::{AsyncIterator, prelude::*};
    ///
    /// struct MyAsyncIter;
    ///
    /// impl AsyncIterator for MyAsyncIter {
    ///     type Item = u32;
    ///
    ///     fn poll_next(
    ///         self: Pin<&mut Self>,
    ///         _cx: &mut Context<'_>
    ///     ) -> Poll<Option<Self::Item>> {
    ///         Ready(Some(1))
    ///     }
    /// }
    ///
    /// async fn run() {
    ///     let mut count = 0;
    ///     for (i, prepare) in MyAsyncIter.into_iter().await.take(3) {
    ///         count += i;
    ///         prepare.await;
    ///     }
    ///     assert_eq!(count, 3);
    /// }
    ///
    /// pasts::block_on(run());
    /// ```
    fn into_iter(self) -> IntoAsyncIterFuture<Self>
    where
        Self: Sized + Unpin,
    {
        IntoAsyncIterFuture(Some(self))
    }
}

impl<I: AsyncIterator + Unpin + ?Sized> AsyncIterator for &mut I {
    type Item = I::Item;

    #[inline]
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        Pin::new(&mut **self).poll_next(cx)
    }
}

impl<I: AsyncIterator + Unpin> AsyncIterator for [I] {
    type Item = (usize, Option<I::Item>);

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        for (i, mut this) in self.iter_mut().enumerate() {
            if let Ready(value) = Pin::new(&mut this).poll_next(cx) {
                return Ready(Some((i, value)));
            }
        }
        Pending
    }
}

#[allow(missing_debug_implementations)]
pub struct IterNextFuture<'a, I>(&'a mut (dyn AsyncIterator<Item = I> + Unpin));

impl<I> Future for IterNextFuture<'_, I> {
    type Output = Option<I>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().0).poll_next(cx)
    }
}

#[derive(Debug)]
pub struct IntoAsyncIterFuture<I: AsyncIterator + Unpin>(Option<I>);

impl<I: AsyncIterator + Unpin> Future for IntoAsyncIterFuture<I>
where
    I::Item: Unpin,
{
    type Output = AsyncIter<I>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self;
        let mut iter = this.0.take().expect("Future awaited after completion");

        if let Ready(item) = Pin::new(&mut iter).poll_next(cx) {
            Ready(AsyncIter((iter, item).into()))
        } else {
            this.0 = Some(iter);
            Pending
        }
    }
}

/// Type returned from [`AsyncIterator::into_iter`]
#[derive(Debug)]
pub struct AsyncIter<I: AsyncIterator>(RefCell<(I, Option<I::Item>)>)
where
    I::Item: Unpin;

impl<I: AsyncIterator + Unpin> AsyncIter<I>
where
    I::Item: Unpin,
{
    /// Create a new `AsyncIter` from an `AsyncIterator`.
    pub fn new(async_iterator: I) -> Self {
        Self((async_iterator, None).into())
    }

    /// Prepare iterator's next value.
    pub async fn prepare(&mut self) {
        let this = self.0.get_mut();
        this.1 = this.0.next().await;
    }
}

impl<T: Unpin, F> AsyncIter<PollNextFn<T, F>>
where
    F: FnMut(&mut Context<'_>) -> Poll<Option<T>> + Unpin,
{
    /// Create a new `AsyncIter` from a function.
    ///
    /// ```
    /// use pasts::{Loop, AsyncIter, prelude::*};
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
    ///     let past = AsyncIter::from_fn(|_cx| {
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
    pub fn from_fn(poll_next_fn: F) -> Self {
        Self::new(PollNextFn(poll_next_fn))
    }
}

#[derive(Debug)]
pub struct Prepare<'a, I>(RefMut<'a, (I, Option<I::Item>)>)
where
    I: AsyncIterator + Unpin;

impl<I: AsyncIterator + Unpin> Future for Prepare<'_, I> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if let Ready(output) = Pin::new(&mut (*self.0).0).poll_next(cx) {
            (*self.0).1 = output;
            Ready(())
        } else {
            Pending
        }
    }
}

impl<'a, I: AsyncIterator + Unpin> Iterator for &'a AsyncIter<I>
where
    I::Item: Unpin,
{
    type Item = (I::Item, Prepare<'a, I>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut this = self.0.borrow_mut();
        this.1.take().map(|item| (item, Prepare(this)))
    }
}

impl<I: AsyncIterator + Unpin> AsyncIterator for AsyncIter<I>
where
    I::Item: Unpin,
{
    type Item = I::Item;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if let Some(queued) = this.0.get_mut().1.take() {
            Ready(Some(queued))
        } else {
            Pin::new(&mut this.0.get_mut().0).poll_next(cx)
        }
    }
}

impl<I: Unpin + Iterator> AsyncIter<IterAsyncIterator<I>>
where
    I::Item: Unpin,
{
    /// Converts an iterator into an async iterator.
    #[allow(clippy::should_implement_trait)] // Trait not general enough
    pub fn from_iter(iter: I) -> Self {
        Self((IterAsyncIterator(iter.fuse()), None).into())
    }
}

impl<I: Unpin + Iterator> AsyncIter<FutureIterAsyncIterator<I>>
where
    I::Item: Future + Unpin,
    <I::Item as Future>::Output: Unpin,
{
    /// Converts an iterator of futures into an async iterator.
    pub fn from(iter: I) -> Self {
        Self((FutureIterAsyncIterator(iter.fuse(), None), None).into())
    }
}

impl<I: Unpin + Iterator> AsyncIter<BoxFutureIterAsyncIterator<I>>
where
    I::Item: Future,
    <I::Item as Future>::Output: Unpin,
{
    /// Converts an iterator of `!Unpin` futures into an async iterator.
    ///
    /// Requires a non-ZST allocator
    pub fn from_pin(iter: I) -> Self {
        Self((BoxFutureIterAsyncIterator(iter.fuse(), None), None).into())
    }
}

#[derive(Debug)]
pub struct PollNextFn<T: Unpin, F>(F)
where
    F: FnMut(&mut Context<'_>) -> Poll<Option<T>> + Unpin;

impl<T: Unpin, F> AsyncIterator for PollNextFn<T, F>
where
    F: FnMut(&mut Context<'_>) -> Poll<Option<T>> + Unpin,
{
    type Item = T;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<T>> {
        self.0(cx)
    }
}

#[derive(Debug)]
pub struct IterAsyncIterator<I: Iterator + Unpin>(Fuse<I>);

impl<I: Iterator + Unpin> AsyncIterator for IterAsyncIterator<I> {
    type Item = I::Item;

    fn poll_next(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        Ready(self.0.next())
    }
}

#[derive(Debug)]
pub struct FutureIterAsyncIterator<I>(Fuse<I>, Option<I::Item>)
where
    I::Item: Future + Unpin,
    I: Iterator + Unpin;

impl<I: Iterator + Unpin> AsyncIterator for FutureIterAsyncIterator<I>
where
    I::Item: Future + Unpin,
{
    type Item = <I::Item as Future>::Output;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if let Some(ref mut future) = &mut self.1 {
            if let Ready(output) = Pin::new(future).poll(cx) {
                self.1 = self.0.next();
                Ready(Some(output))
            } else {
                Pending
            }
        } else if let Some(next) = self.0.next() {
            self.1 = Some(next);
            self.poll_next(cx)
        } else {
            Ready(None)
        }
    }
}

#[derive(Debug)]
pub struct BoxFutureIterAsyncIterator<I>(Fuse<I>, Option<Pin<Box<I::Item>>>)
where
    I: Iterator + Unpin,
    I::Item: Future;

impl<I: Iterator + Unpin> AsyncIterator for BoxFutureIterAsyncIterator<I>
where
    I::Item: Future,
{
    type Item = <I::Item as Future>::Output;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.get_mut();
        let (iter, fut) = (&mut this.0, &mut this.1);
        if let Some(ref mut future) = fut {
            let mut future = future;
            let poll = Pin::new(&mut future).poll(cx);
            if let Ready(output) = poll {
                if let Some(next) = iter.next() {
                    future.set(next);
                } else {
                    *fut = None;
                }
                Ready(Some(output))
            } else {
                Pending
            }
        } else if let Some(next) = iter.next() {
            *fut = Some(Box::pin(next));
            Pin::new(&mut this).poll_next(cx)
        } else {
            Ready(None)
        }
    }
}

impl<P, T: AsyncIterator + Unpin + ?Sized> AsyncIterator for Pin<P>
where
    P: Unpin + core::ops::DerefMut<Target = T>,
{
    type Item = T::Item;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        Pin::new(&mut **self.get_mut()).poll_next(cx)
    }
}
