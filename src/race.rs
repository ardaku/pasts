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

/// Trait for attaching state to a future type (allows type nesting).
pub trait Stateful {
    type State;

    fn state(&mut self) -> &mut Self::State;
}

/// Future that is never ready.
#[derive(Debug)]
pub struct Never<'a, T, S>(&'a mut S, PhantomData<*mut T>);

impl<T, S> Future for Never<'_, T, S> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<T> {
        Poll::Pending
    }
}

impl<T, S> Stateful for Never<'_, T, S> {
    type State = S;

    fn state(&mut self) -> &mut S {
        self.0
    }
}

/// An asynchronous event.
struct EventSlice<F, G, V, S, T, Z>
where
    F: Future<Output = Poll<T>> + Stateful<State = S> + Unpin,
    G: for<'a> Fn(&'a mut S) -> &'a mut [Z] + Unpin,
    Z: Future<Output = V> + Unpin,
{
    // The wrapped stateful future.
    other: F,
    // Future getter closure/function.
    future: G,
    // Callback function
    event: fn(&mut S, usize, V) -> Poll<T>,
}

impl<F, G, V, S, T, Z> Future for EventSlice<F, G, V, S, T, Z>
where
    F: Future<Output = Poll<T>> + Stateful<State = S> + Unpin,
    G: for<'a> Fn(&'a mut S) -> &'a mut [Z] + Unpin,
    Z: Future<Output = V> + Unpin,
{
    type Output = Poll<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Poll<T>> {
        let this = self.get_mut();
        if let Poll::Ready(output) = Pin::new(&mut this.other).poll(cx) {
            return Poll::Ready(output);
        }
        for (i, f) in (this.future)(this.other.state()).iter_mut().enumerate() {
            if let Poll::Ready(out) = Pin::new(f).poll(cx) {
                return Poll::Ready((this.event)(this.other.state(), i, out));
            }
        }
        Poll::Pending
    }
}

impl<F, G, V, S, T, Z> Stateful for EventSlice<F, G, V, S, T, Z>
where
    F: Future<Output = Poll<T>> + Stateful<State = S> + Unpin,
    G: for<'a> Fn(&'a mut S) -> &'a mut [Z] + Unpin,
    Z: Future<Output = V> + Unpin,
{
    type State = S;

    fn state(&mut self) -> &mut S {
        self.other.state()
    }
}

/// An asynchronous event.
struct Event<F, G, V, S, T, Z>
where
    F: Future<Output = Poll<T>> + Stateful<State = S> + Unpin,
    G: for<'a> Fn(&'a mut S) -> &'a mut Z + Unpin,
    Z: Future<Output = V> + Unpin,
{
    // The wrapped stateful future.
    wrapped: F,
    // Future getter closure/function.
    future: G,
    // Callback function
    callback: fn(&mut S, V) -> Poll<T>,
}

impl<F, G, V, S, T, Z> Future for Event<F, G, V, S, T, Z>
where
    F: Future<Output = Poll<T>> + Stateful<State = S> + Unpin,
    G: for<'a> Fn(&'a mut S) -> &'a mut Z + Unpin,
    Z: Future<Output = V> + Unpin,
{
    type Output = Poll<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Poll<T>> {
        let this = self.get_mut();
        if let Poll::Ready(output) = Pin::new(&mut this.wrapped).poll(cx) {
            Poll::Ready(output)
        } else if let Poll::Ready(output) =
            Pin::new((this.future)(this.wrapped.state())).poll(cx)
        {
            Poll::Ready((this.callback)(this.wrapped.state(), output))
        } else {
            Poll::Pending
        }
    }
}

impl<F, G, V, S, T, Z> Stateful for Event<F, G, V, S, T, Z>
where
    F: Future<Output = Poll<T>> + Stateful<State = S> + Unpin,
    G: for<'a> Fn(&'a mut S) -> &'a mut Z + Unpin,
    Z: Future<Output = V> + Unpin,
{
    type State = S;

    fn state(&mut self) -> &mut S {
        self.wrapped.state()
    }
}

/// An asynchronous event loop.
#[derive(Debug)]
pub struct Loop<S, T, F>(F)
where
    F: Future<Output = Poll<T>> + Stateful<State = S> + Unpin;

impl<'a, S, T> Loop<S, T, Never<'a, Poll<T>, S>> {
    /// Create a new asynchronous event loop.
    pub fn new(state: &'a mut S) -> Self {
        Self(Never(state, PhantomData))
    }
}

impl<S, T, F> Loop<S, T, F>
where
    F: Future<Output = Poll<T>> + Stateful<State = S> + Unpin,
{
    /// Add an asynchronous event to the event loop.
    pub fn when<G, V, Z>(
        self,
        f: G,
        c: fn(&mut S, V) -> Poll<T>,
    ) -> Loop<S, T, impl Future<Output = Poll<T>> + Stateful<State = S>>
    where
        G: for<'a> Fn(&'a mut S) -> &'a mut Z + Unpin,
        Z: Future<Output = V> + Unpin,
    {
        Loop(Event {
            wrapped: self.0,
            future: f,
            callback: c,
        })
    }

    /// Add asynchronous event polling from a slice to the event loop.
    pub fn poll<G, V, Z>(
        self,
        f: G,
        c: fn(&mut S, usize, V) -> Poll<T>,
    ) -> Loop<S, T, impl Future<Output = Poll<T>> + Stateful<State = S>>
    where
        G: for<'a> Fn(&'a mut S) -> &'a mut [Z] + Unpin,
        Z: Future<Output = V> + Unpin,
    {
        Loop(EventSlice {
            other: self.0,
            future: f,
            event: c,
        })
    }
}

impl<S, T, F> Future for Loop<S, T, F>
where
    F: Future<Output = Poll<T>> + Stateful<State = S> + Unpin,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        while let Poll::Ready(output) = Pin::new(&mut self.0).poll(cx) {
            if let Poll::Ready(output) = output {
                return Poll::Ready(output);
            }
        }
        Poll::Pending
    }
}
