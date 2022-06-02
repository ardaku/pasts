// Copyright © 2019-2022 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::{prelude::*, Notifier};

pub trait Stateful<S, T>: Unpin {
    fn state(&mut self) -> &mut S;

    fn poll(&mut self, _: &mut TaskCx<'_>) -> Poll<Poll<T>> {
        Pending
    }
}

#[derive(Debug)]
pub struct Never<'a, S>(&'a mut S);

impl<S, T> Stateful<S, T> for Never<'_, S> {
    fn state(&mut self) -> &mut S {
        self.0
    }
}

/// Composable asynchronous event loop.
///
/// # Selecting on Futures:
/// Select first completed future.
///
/// ```rust
#[doc = include_str!("../examples/slices.rs")]
/// ```
/// 
/// # Task spawning
/// Spawns tasks in a [`Vec`], and removes them as they complete.
/// ```rust
#[doc = include_str!("../examples/tasks.rs")]
/// ```
///
#[derive(Debug)]
pub struct Race<S: Unpin, T, F: Stateful<S, T>> {
    other: F,
    _phantom: core::marker::PhantomData<(S, T)>,
}

impl<'a, S: Unpin, T> Race<S, T, Never<'a, S>> {
    /// Create an empty event loop.

    pub fn new(state: &'a mut S) -> Self {
        let other = Never(state);
        let _phantom = core::marker::PhantomData;

        Race { other, _phantom }
    }
}

impl<S: Unpin, T, F: Stateful<S, T>> Race<S, T, F> {
    /// Register an event handler.
    pub fn on<N>(
        self,
        past: N,
        then: fn(&mut S, N::Event) -> Poll<T>,
    ) -> Race<S, T, impl Stateful<S, T>>
    where
        N: Notifier + Unpin,
    {
        let other = self.other;
        let _phantom = core::marker::PhantomData;
        let other = Join { other, past, then };

        Race { other, _phantom }
    }
}

impl<S: Unpin, T: Unpin, F: Stateful<S, T>> Future for Race<S, T, F> {
    type Output = T;

    #[inline]

    fn poll(mut self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<T> {
        while let Ready(output) = Pin::new(&mut self.other).poll(cx) {
            if let Ready(output) = output {
                return Ready(output);
            }
        }

        Pending
    }
}

struct Join<S, T, E, F: Stateful<S, T>, N: Notifier<Event = E>> {
    other: F,
    past: N,
    then: fn(&mut S, E) -> Poll<T>,
}

impl<S, T, E, F, N> Stateful<S, T> for Join<S, T, E, F, N>
where
    F: Stateful<S, T>,
    N: Notifier<Event = E> + Unpin,
{
    #[inline]
    fn state(&mut self) -> &mut S {
        self.other.state()
    }

    #[inline]
    fn poll(&mut self, cx: &mut TaskCx<'_>) -> Poll<Poll<T>> {
        let poll = Pin::new(&mut self.past).poll_next(cx);

        if let Ready(out) = poll.map(|x| (self.then)(self.other.state(), x)) {
            Ready(out)
        } else {
            self.other.poll(cx)
        }
    }
}
