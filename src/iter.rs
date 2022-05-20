// Copyright © 2019-2022 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::prelude::*;

/// Trait for asynchronous iteration.
///
/// All provided methods produce [`Future`]s.
///
/// Polyfill for [`AsyncIterator`](core::async_iter::AsyncIterator), with a
/// few deviations:
///  - [`poll_next()`](AsyncIterator::poll_next) takes `&mut Self` instead of
///    `Pin<&mut Self>`
///  - [`into_iter()`](AsyncIterator::into_iter) replaces duplicating iterator
///    methods
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
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;

    /// Get the next
    /// [`Future`]`<`[`Output`](core::future::Future::Output)`=`[`Option`]`<`[`Self::Item`]`>>`
    fn next(&mut self) -> IterNextFuture<'_, Self> {
        IterNextFuture(self)
    }

    /// Convert this `AsyncIterator` into an [`AsyncIter`].
    fn into_iter(self) -> IntoAsyncIterFuture<Self>
    where
        Self: Sized + Unpin,
    {
        IntoAsyncIterFuture(Some(self))
    }
}

impl<I: AsyncIterator + ?Sized> AsyncIterator for &mut I {
    type Item = I::Item;

    #[inline]
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        (**self).poll_next(cx)
    }
}

#[derive(Debug)]
pub struct IterNextFuture<'a, I: AsyncIterator + ?Sized>(&'a mut I);

impl<I: AsyncIterator + ?Sized> Future for IterNextFuture<'_, I> {
    type Output = Option<I::Item>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_mut().0.poll_next(cx)
    }
}

#[derive(Debug)]
pub struct IntoAsyncIterFuture<I: AsyncIterator + Unpin>(Option<I>);

impl<I: AsyncIterator + Unpin> Future for IntoAsyncIterFuture<I> {
    type Output = AsyncIter<I>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self;
        let err = "Future awaited after completion";
        let mut iter = this.0.take().expect(err);

        if let Ready(item) = iter.poll_next(cx) {
            Ready(AsyncIter(iter, item))
        } else {
            this.0 = Some(iter);
            Pending
        }
    }
}

/// Type returned from [`AsyncIterator::into_iter`]
#[derive(Debug)]
pub struct AsyncIter<I: AsyncIterator>(I, Option<I::Item>);

impl<I: AsyncIterator> AsyncIter<I> {
    /// Prepare iterator's next value.
    ///
    /// Should be called at the end of a `for` loop, and before any occurances
    /// of `continue`.  Failure to do so may cause bugs.
    pub async fn prepare(&mut self) {
        self.1 = self.0.next().await;
    }
}

impl<I: AsyncIterator> Iterator for AsyncIter<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.1.take()
    }
}

impl<I: AsyncIterator> AsyncIterator for AsyncIter<I> {
    type Item = I::Item;

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(queued) = self.1.take() {
            Ready(Some(queued))
        } else {
            self.0.poll_next(cx)
        }
    }
}
