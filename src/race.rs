// Pasts
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

#![allow(unsafe_code)] // FIXME: Move to util

use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

struct MultiFuture<S, F, L, G, U>
where
    F: Future<Output = U> + Unpin,
    G: Future<Output = L> + Unpin,
{
    state: *mut S,
    future: *mut F,
    other: G,
    translator: fn(&mut S, U) -> L,
}

impl<S, F, L, G, U> Future for MultiFuture<S, F, L, G, U>
where
    F: Future<Output = U> + Unpin,
    G: Future<Output = L> + Unpin,
{
    type Output = L;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        match Pin::new(&mut self.other).poll(cx) {
            Poll::Pending => match Pin::new(unsafe { &mut *self.future }).poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(output) => Poll::Ready((self.translator)(
                    unsafe { &mut *self.state }, output
                )),
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
pub struct Race<'a, S, F, T>
where
    F: Future<Output = T> + Unpin, T: Unpin
{
    state: *mut S,
    future: F,
    _phantom: PhantomData<&'a mut S>,
}

impl<'a, S, T: Unpin> Race<'a, S, Never<T>, T> {
    /// Create an empty Race.
    pub fn new<F, X>(state: &'a mut S, builder: F) -> Race<'a, S, X, T>
        where F: Fn(&'a mut S, Self) -> Race<'a, S, X, T>, X: Future<Output = T> + Unpin
    {
        let race = Self {
            state: state,
            future: Never(PhantomData),
            _phantom: PhantomData,
        };
        builder(state, race)
    }
}

impl<'a, S, F, T> Race<'a, S, F, T>
where
    F: Future<Output = T> + Unpin, T: Unpin
{
    /// Add an asynchronous event.
    pub fn when<E, U>(
        self,
        future: &mut E,
        event: fn(&mut S, U) -> T,
    ) -> Race<'a, S, impl Future<Output = T> + Unpin, T>
    where
        E: Future<Output = U> + Unpin,
    {
        Race {
            state: self.state,
            future: MultiFuture {
                state: self.state,
                future,
                translator: event,
                other: self.future,
            },
            _phantom: PhantomData,
        }
    }
}

impl<S, F, T> Future for Race<'_, S, F, T>
where
    F: Future<Output = T> + Unpin, T: Unpin
{
    type Output = T;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        Pin::new(&mut self.future).poll(cx)
    }
}
