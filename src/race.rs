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

use crate::past::Past;
use crate::util::{MultiFuture, PastFuture};

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

/// Asynchonous event loop builder.
#[derive(Debug)]
pub struct LoopBuilder<S, F, T>
where
    F: Future<Output = T> + Stateful<S> + Unpin,
    T: Unpin,
{
    future: F,
    _phantom: PhantomData<*mut S>,
}

impl<S, F, T> LoopBuilder<S, F, T>
where
    F: Future<Output = T> + Stateful<S> + Unpin,
    T: Unpin,
{
    /// Add an asynchronous event.
    pub fn when<E, U>(
        self,
        future: &mut E,
        event: fn(&mut S, U) -> T,
    ) -> LoopBuilder<S, MultiFuture<S, E, T, F, U>, T>
    where
        E: Future<Output = U> + Unpin,
    {
        LoopBuilder {
            future: MultiFuture {
                future,
                translator: event,
                other: self.future,
            },
            _phantom: PhantomData,
        }
    }

    /// Add an asynchronous event polling from a list (either a Vec or array) of
    /// futures.
    #[allow(clippy::type_complexity)]
    pub fn poll<E, U>(
        self,
        future: &mut E,
        event: fn(&mut S, U) -> T,
    ) -> LoopBuilder<S, MultiFuture<S, PastFuture<U, E>, T, F, U>, T>
    where
        E: Past<U>,
    {
        let future = PastFuture::with(future);
        LoopBuilder {
            future: MultiFuture {
                future,
                translator: event,
                other: self.future,
            },
            _phantom: PhantomData,
        }
    }
}

struct RaceFuture<T: Unpin, L: Loop<T>>(L, PhantomData<*mut T>);

impl<T: Unpin, L: Loop<T>> Future for RaceFuture<T, L> {
    type Output = Poll<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Poll<T>> {
        self.0.poll(cx)
    }
}

pub trait Seal {}

/// Asynchonous event loop executor.
pub type EventLoop<S, O> = LoopBuilder<S, Never<Poll<O>, S>, Poll<O>>;

/// An asynchonous event loop.
pub trait Loop<T>: Unpin + Seal {
    #[doc(hidden)]
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Poll<T>>;
}

impl<S, F, T> Seal for LoopBuilder<S, F, Poll<T>>
where
    F: Future<Output = Poll<T>> + Stateful<S> + Unpin,
    T: Unpin,
{
}

impl<S, F, T> Loop<T> for LoopBuilder<S, F, Poll<T>>
where
    F: Future<Output = Poll<T>> + Stateful<S> + Unpin,
    T: Unpin,
{
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Poll<T>> {
        Pin::new(&mut self.future).poll(cx)
    }
}

impl<S, O: Unpin> EventLoop<S, O> {
    /// Execute multiple asynchronous tasks at once in an event loop.
    pub async fn run<F, X>(state: &mut S, looper: F) -> O
    where
        F: Fn(&mut S, LoopBuilder<S, Never<Poll<O>, S>, Poll<O>>) -> X,
        X: Loop<O>,
        O: Unpin,
    {
        loop {
            let race = LoopBuilder {
                future: Never(state, PhantomData),
                _phantom: PhantomData,
            };
            if let Poll::Ready(output) =
                RaceFuture(looper(state, race), PhantomData).await
            {
                break output;
            }
        }
    }
}
