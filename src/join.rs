// Copyright © 2019-2023 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// - MIT License (https://mit-license.org/)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::{prelude::*, Notify};

pub trait Stateful<S, T>: Unpin {
    fn state(&mut self) -> &mut S;

    fn poll(&mut self, _: &mut Task<'_>) -> Poll<Poll<T>> {
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
/// Spawns tasks in a [`Vec`](alloc::vec::Vec), and removes them as they complete.
/// ```rust
#[doc = include_str!("../examples/tasks.rs")]
/// ```
#[derive(Debug)]
pub struct Join<S: Unpin, T, F: Stateful<S, T>> {
    other: F,
    _phantom: core::marker::PhantomData<(S, T)>,
}

impl<'a, S: Unpin, T> Join<S, T, Never<'a, S>> {
    /// Create an empty event loop.
    pub fn new(state: &'a mut S) -> Self {
        let other = Never(state);
        let _phantom = core::marker::PhantomData;

        Join { other, _phantom }
    }
}

impl<S: Unpin, T, F: Stateful<S, T>> Join<S, T, F> {
    /// Register an event handler.
    pub fn on<N: Notify + Unpin + ?Sized>(
        self,
        noti: impl for<'a> FnMut(&'a mut S) -> &'a mut N + Unpin,
        then: fn(&mut S, N::Event) -> Poll<T>,
    ) -> Join<S, T, impl Stateful<S, T>> {
        let other = self.other;
        let _phantom = core::marker::PhantomData;
        let other = Joiner { other, noti, then };

        Join { other, _phantom }
    }
}

impl<S: Unpin, T: Unpin, F: Stateful<S, T>> Future for Join<S, T, F> {
    type Output = T;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, t: &mut Task<'_>) -> Poll<T> {
        while let Ready(output) = Pin::new(&mut self.other).poll(t) {
            if let Ready(output) = output {
                return Ready(output);
            }
        }

        Pending
    }
}

struct Joiner<S, T, E, F: Stateful<S, T>, P> {
    other: F,
    noti: P,
    then: fn(&mut S, E) -> Poll<T>,
}

impl<S, T, E, F, N, P> Stateful<S, T> for Joiner<S, T, E, F, P>
where
    F: Stateful<S, T>,
    N: Notify<Event = E> + Unpin + ?Sized,
    P: for<'a> FnMut(&'a mut S) -> &'a mut N + Unpin,
{
    #[inline]
    fn state(&mut self) -> &mut S {
        self.other.state()
    }

    #[inline]
    fn poll(&mut self, t: &mut Task<'_>) -> Poll<Poll<T>> {
        let state = self.other.state();
        let poll = Pin::new((self.noti)(state)).poll_next(t);

        if let Ready(out) = poll.map(|x| (self.then)(state, x)) {
            Ready(out)
        } else {
            self.other.poll(t)
        }
    }
}
