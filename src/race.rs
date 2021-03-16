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
use core::pin::Pin;
use core::task::{Context, Poll};
use core::marker::PhantomData;

#[derive(Debug)]
pub struct Translator<
    T,
    U,
    N: Fn(U) -> T + Unpin,
    F: Future<Output = U> + Unpin,
>(N, F);

impl<T, U, N: Fn(U) -> T + Unpin, F: Future<Output = U> + Unpin> Future
    for Translator<T, U, N, F>
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let a = Pin::new(&mut self.1);
        match a.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(x) => Poll::Ready(self.0(x)),
        }
    }
}

#[derive(Debug)]
pub struct Future2<
    'a,
    T,
    A: Future<Output = T> + ?Sized + Unpin,
    B: Future<Output = T> + Unpin,
>(&'a mut A, B);

/// A method on [`Future`](std::future::Future)s that runs futures in
/// parallel and returns the result of the first future to complete.
///
/// The futures must implement [`Unpin`](std::marker::Unpin).
pub trait Race<T>: Future<Output = T> + Unpin {
    /// Race with another future.  Returns a [`Future`](std::future::Future).
    fn race<F: Future<Output = T> + Unpin>(
        &mut self,
        other: F,
    ) -> Future2<'_, T, Self, F> {
        Future2(self, other)
    }

    /// Listen for events from a future.
    /// Returns a [`Future`](std::future::Future).
    fn when<U, F: Future<Output = U> + Unpin, N: Fn(U) -> T + Unpin>(
        &mut self,
        event: N,
        other: F,
    ) -> Future2<'_, T, Self, Translator<T, U, N, F>> {
        Future2(self, Translator(event, other))
    }
}

impl<T, X> Race<T> for X where X: Future<Output = T> + Unpin {}

impl<
        T,
        A: Future<Output = T> + ?Sized + Unpin,
        B: Future<Output = T> + Unpin,
    > Future for Future2<'_, T, A, B>
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let a = Pin::new(&mut self.0);

        match a.poll(cx) {
            Poll::Pending => {
                let b = Pin::new(&mut self.1);
                b.poll(cx)
            }
            x => x,
        }
    }
}

#[derive(Debug)]
pub struct Never<T: Unpin>(PhantomData<T>);

impl<T: Unpin> Future for Never<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<T> {
        Poll::Pending
    }
}

/// A loop that listens for events asynchronously.
#[derive(Debug)]
pub struct Loop<F: Future<Output = T> + Unpin, T, S: Unpin>(F, PhantomData<S>);

impl<T: Unpin, S: Unpin> Default for Loop<Never<T>, T, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Unpin, S: Unpin> Loop<Never<T>, T, S> {
    /// Create an empty loop.
    pub fn new() -> Self {
        Loop(Never(PhantomData), PhantomData)
    }
}

impl<F: Future<Output = T> + Unpin, T: Unpin, S: Unpin> Loop<F, T, S> {
    /// Add an asynchronous event.
    #[allow(clippy::type_complexity)]
    pub fn when<
        U: Unpin,
        G: Future<Output = U> + Unpin,
        N: Fn(U) -> T + Unpin,
    >(
        &mut self,
        future: G,
        event: N,
    ) -> Loop<Future2<'_, T, F, Translator<T, U, N, G>>, T, S> {
        Loop(self.0.when(event, future), PhantomData)
    }

    /// Attach a state to the `Loop`.
    pub fn attach(&mut self, state: S) -> Attach<'_, F, T, S> {
        Attach(self, state)
    }
}

#[derive(Debug)]
pub struct Attach<'a, F: Future<Output = T> + Unpin, T: Unpin, S: Unpin>(&'a mut Loop<F, T, S>, S);

impl<F: Future<Output = T> + Unpin, T: Unpin, S: Unpin> Future for Attach<'_, F, T, S> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let this = self.get_mut();
        Pin::new(&mut this.0.0).poll(cx)
    }
}
