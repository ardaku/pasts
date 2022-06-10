// Copyright © 2019-2022 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// - MIT License (https://mit-license.org/)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::{prelude::*, Notifier};

pub trait Stateful<S, T>: Unpin {
    fn state(&mut self) -> &mut S;

    fn poll(&mut self, _: &mut Exec<'_>) -> Poll<Poll<T>> {
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
/// # extern crate alloc;
/// # #[allow(unused_imports)]
/// # use self::main::*;
/// # mod main {
#[doc = include_str!("../examples/slices/src/main.rs")]
/// #     pub(super) mod main {
/// #         pub(in crate) async fn main(executor: pasts::Executor){
/// #             super::main(&executor).await
/// #         }
/// #     }
/// # }
/// # fn main() {
/// #     let executor = pasts::Executor::default();
/// #     executor.spawn(Box::pin(self::main::main::main(executor.clone())));
/// # }
/// ```
/// 
/// # Task spawning
/// Spawns tasks in a [`Vec`], and removes them as they complete.
/// ```rust
/// # extern crate alloc;
/// # #[allow(unused_imports)]
/// # use self::main::*;
/// # mod main {
#[doc = include_str!("../examples/tasks/src/main.rs")]
/// #     pub(super) mod main {
/// #         pub(in crate) async fn main(executor: pasts::Executor){
/// #             super::main(&executor).await
/// #         }
/// #     }
/// # }
/// # fn main() {
/// #     let executor = pasts::Executor::default();
/// #     executor.spawn(Box::pin(self::main::main::main(executor.clone())));
/// # }
/// ```
/// 
/// <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
/// <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
/// <script>hljs.highlightAll();</script>
/// <style> code.hljs { background-color: #000B; } </style>
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
    pub fn on<N: Notifier + Unpin + ?Sized>(
        self,
        past: impl for<'a> FnMut(&'a mut S) -> &'a mut N + Unpin,
        then: fn(&mut S, N::Event) -> Poll<T>,
    ) -> Join<S, T, impl Stateful<S, T>> {
        let other = self.other;
        let _phantom = core::marker::PhantomData;
        let other = Joiner { other, past, then };

        Join { other, _phantom }
    }
}

impl<S: Unpin, T: Unpin, F: Stateful<S, T>> Future for Join<S, T, F> {
    type Output = T;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, e: &mut Exec<'_>) -> Poll<T> {
        while let Ready(output) = Pin::new(&mut self.other).poll(e) {
            if let Ready(output) = output {
                return Ready(output);
            }
        }

        Pending
    }
}

struct Joiner<S, T, E, F: Stateful<S, T>, P> {
    other: F,
    past: P,
    then: fn(&mut S, E) -> Poll<T>,
}

impl<S, T, E, F, N, P> Stateful<S, T> for Joiner<S, T, E, F, P>
where
    F: Stateful<S, T>,
    N: Notifier<Event = E> + Unpin + ?Sized,
    P: for<'a> FnMut(&'a mut S) -> &'a mut N + Unpin,
{
    #[inline]
    fn state(&mut self) -> &mut S {
        self.other.state()
    }

    #[inline]
    fn poll(&mut self, e: &mut Exec<'_>) -> Poll<Poll<T>> {
        let state = self.other.state();
        let poll = Pin::new((self.past)(state)).poll_next(e);

        if let Ready(out) = poll.map(|x| (self.then)(state, x)) {
            Ready(out)
        } else {
            self.other.poll(e)
        }
    }
}
