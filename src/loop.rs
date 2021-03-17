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

pub trait StatefulFuture<S, T> {
    fn poll(&mut self, state: &mut S, cx: &mut Context<'_>) -> Poll<T>;
}

struct MultiFuture<S, F, O, L, E>
where
    F: Future<Output = O> + Unpin,
    E: StatefulFuture<S, L>,
{
    future: F,
    translator: fn(&mut S, O) -> Poll<L>,
    other: E,
}

impl<S, F, O, L, E> StatefulFuture<S, L> for MultiFuture<S, F, O, L, E>
where
    F: Future<Output = O> + Unpin,
    E: StatefulFuture<S, L>,
{
    fn poll(&mut self, state: &mut S, cx: &mut Context<'_>) -> Poll<L> {
        match self.other.poll(state, cx) {
            Poll::Pending => loop {
                match Pin::new(&mut self.future).poll(cx) {
                    Poll::Pending => break Poll::Pending,
                    Poll::Ready(x) => {
                        match (self.translator)(state, x) {
                            Poll::Ready(x) => break Poll::Ready(x),
                            Poll::Pending => { /* continue */ }
                        }
                    }
                }
            },
            x => x,
        }
    }
}

#[derive(Debug)]
pub struct Never<S, L>(PhantomData<(S, L)>);

impl<S, L> StatefulFuture<S, L> for Never<S, L> {
    fn poll(&mut self, _: &mut S, _: &mut Context<'_>) -> Poll<L> {
        Poll::Pending
    }
}

/// A future that listens for events asynchronously in a loop.
#[derive(Debug)]
pub struct Loop<T: Unpin, F: StatefulFuture<S, T>, S: Unpin>(
    F,
    S,
    PhantomData<T>,
);

impl<T: Unpin, S: Unpin> Loop<T, Never<S, T>, S> {
    /// Create an empty loop.
    pub fn new(state: S) -> Self {
        Loop(Never(PhantomData), state, PhantomData)
    }
}

impl<T: Unpin, F: StatefulFuture<S, T>, S: Unpin> Loop<T, F, S> {
    /// Add an asynchronous event.
    pub fn when<U, G: Future<Output = U> + Unpin>(
        self,
        future: G,
        event: fn(&mut S, U) -> Poll<T>,
    ) -> Loop<T, impl StatefulFuture<S, T>, S> {
        let multifuture = MultiFuture {
            future,
            translator: event,
            other: self.0,
        };

        Loop(multifuture, self.1, PhantomData)
    }
}

impl<T: Unpin, F: StatefulFuture<S, T> + Unpin, S: Unpin> Future
    for Loop<T, F, S>
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let this = self.get_mut();
        this.0.poll(&mut this.1, cx)
    }
}
