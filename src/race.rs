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

use crate::past::Past;

#[allow(missing_debug_implementations)]
pub struct MultiFuture<S, F, L, G, U>
where
    F: Future<Output = U> + Unpin,
    G: Future<Output = L> + Stateful<S> + Unpin,
{
    future: *mut F,
    other: G,
    translator: fn(&mut S, U) -> L,
}

impl<S, F, L, G, U> Stateful<S> for MultiFuture<S, F, L, G, U>
where
    F: Future<Output = U> + Unpin,
    G: Future<Output = L> + Stateful<S> + Unpin,
{
    fn state(&mut self) -> *mut S {
        self.other.state()
    }
}

impl<S, F, L, G, U> Future for MultiFuture<S, F, L, G, U>
where
    F: Future<Output = U> + Unpin,
    G: Future<Output = L> + Stateful<S> + Unpin,
{
    type Output = L;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        match Pin::new(&mut self.other).poll(cx) {
            Poll::Pending => {
                match Pin::new(unsafe { &mut *self.future }).poll(cx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(output) => Poll::Ready((self.translator)(
                        unsafe { &mut *self.state() },
                        output,
                    )),
                }
            }
            x => x,
        }
    }
}

#[derive(Debug)]
pub struct Never<T, S>(*mut S, PhantomData<T>);

impl<T, S> Future for Never<T, S> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<T> {
        Poll::Pending
    }
}

impl<T, S> Stateful<S> for Never<T, S> {
    fn state(&mut self) -> *mut S {
        self.0
    }
}

pub trait Stateful<S> {
    fn state(&mut self) -> *mut S;
}

/// A future that returns a closure for the first completed future.
#[derive(Debug)]
pub struct Race<S, F, T>
where
    F: Future<Output = T> + Stateful<S> + Unpin,
    T: Unpin,
{
    future: F,
    _phantom: PhantomData<*mut S>,
}

impl<S, F, T> Race<S, F, T>
where
    F: Future<Output = T> + Stateful<S> + Unpin,
    T: Unpin,
{
    /// Add an asynchronous event.
    pub fn when<E, U>(
        self,
        future: &mut E,
        event: fn(&mut S, U) -> T,
    ) -> Race<S, MultiFuture<S, E, T, F, U>, T>
    where
        E: Future<Output = U> + Unpin,
    {
        Race {
            future: MultiFuture {
                future,
                translator: event,
                other: self.future,
            },
            _phantom: PhantomData,
        }
    }

    /// Add an asynchronous event polling from a list of futures.
    pub fn poll<E, U>(
        self,
        future: &mut E,
        event: fn(&mut S, U) -> T,
    ) -> Race<S, MultiFuture<S, PastFuture<U, E>, T, F, U>, T>
    where
        E: Past<U>,
    {
        let future = PastFuture::with(future);
        Race {
            future: MultiFuture {
                future,
                translator: event,
                other: self.future,
            },
            _phantom: PhantomData,
        }
    }
}

#[repr(transparent)]
#[allow(missing_debug_implementations)]
pub struct PastFuture<U, P: Past<U>>(P, PhantomData<*mut U>);

impl<U, P: Past<U>> PastFuture<U, P> {
    #[allow(trivial_casts)] // Not sure why it thinks it's trivial, is needed.
    fn with(from: &mut P) -> &mut Self {
        unsafe { &mut *(from as *mut _ as *mut Self) }
    }
}

impl<U, P: Past<U>> Future for PastFuture<U, P> {
    type Output = U;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}

struct RaceFuture<T: Unpin, L: Loop<T>>(L, PhantomData<*mut T>);

impl<T: Unpin, L: Loop<T>> Future for RaceFuture<T, L> {
    type Output = Poll<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Poll<T>> {
        self.0.poll(cx)
    }
}

/// Execute multiple asynchronous tasks at once in an event loop.
pub async fn event_loop<S, F, O, X>(state: &mut S, looper: F) -> O
where
    F: Fn(&mut S, Race<S, Never<Poll<O>, S>, Poll<O>>) -> X,
    X: Loop<O>,
    O: Unpin,
{
    loop {
        let race = Race {
            future: Never(state, PhantomData),
            _phantom: PhantomData,
        };
        if let Poll::Ready(output) = RaceFuture(looper(state, race), PhantomData).await {
            break output;
        }
    }
}

/// Asynchonous event loop builder.
pub type LoopBuilder<S, O> = Race<S, Never<Poll<O>, S>, Poll<O>>;

pub trait Loop<T>: Unpin {
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Poll<T>>;
}

impl<S, F, T> Loop<T> for Race<S, F, Poll<T>>
where
    F: Future<Output = Poll<T>> + Stateful<S> + Unpin,
    T: Unpin,
{
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Poll<T>> {
        Pin::new(&mut self.future).poll(cx)
    }
}
