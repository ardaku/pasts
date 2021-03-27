// Pasts
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

struct MultiFuture<S, F, L, G>
where
    F: Future<Output = ()> + Unpin,
    G: Future<Output = fn(&mut S) -> L> + Unpin,
{
    future: F,
    other: G,
    translator: fn(&mut S) -> L,
}

impl<S, F, L, G> Future for MultiFuture<S, F, L, G>
where
    F: Future<Output = ()> + Unpin,
    G: Future<Output = fn(&mut S) -> L> + Unpin,
{
    type Output = fn(&mut S) -> L;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        match Pin::new(&mut self.other).poll(cx) {
            Poll::Pending => match Pin::new(&mut self.future).poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(()) => Poll::Ready(self.translator),
            },
            x => x,
        }
    }
}

#[derive(Debug)]
pub struct Never<T>(PhantomData<T>);

impl<T> Future for Never<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<T> {
        Poll::Pending
    }
}

/// A future that returns a closure for the first completed future.
#[derive(Debug)]
pub struct Race<S, F, T>
where
    F: Future<Output = fn(&mut S) -> T> + Unpin,
{
    future: F,
}

impl<S, T> Default for Race<S, Never<fn(&mut S) -> T>, T> {
    /// Create an empty Race.
    fn default() -> Self {
        Self::new()
    }
}

impl<S, T> Race<S, Never<fn(&mut S) -> T>, T> {
    /// Create an empty Race.
    pub fn new() -> Self {
        Self {
            future: Never(PhantomData),
        }
    }
}

impl<S, F, T> Race<S, F, T>
where
    F: Future<Output = fn(&mut S) -> T> + Unpin,
{
    /// Add an asynchronous event.
    pub fn when<E>(
        self,
        future: E,
        event: fn(&mut S) -> T,
    ) -> Race<S, impl Future<Output = fn(&mut S) -> T> + Unpin, T>
    where
        E: Future<Output = ()> + Unpin,
    {
        Race {
            future: MultiFuture {
                future,
                translator: event,
                other: self.future,
            },
        }
    }
}

impl<S, F, T> Future for Race<S, F, T>
where
    F: Future<Output = fn(&mut S) -> T> + Unpin,
{
    type Output = fn(&mut S) -> T;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        Pin::new(&mut self.future).poll(cx)
    }
}
