// Copyright © 2019-2022 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use alloc::boxed::Box;
use core::{fmt, future::Future, pin::Pin, task::Context};

use crate::{past::Past, prelude::*};

/// Type-erased `?`[`Unpin`] + [`Send`] fused future.
///
/// Usage of this type requires an allocator.
///
/// # Usage
/// `Task`s are useful for when you:
///  - Need to create a list of [`Future`]s.
///  - Want an abstraction over `Pin<Box<dyn Future<Output = O> + Send>>`
///
/// ```rust
/// use pasts::Task;
///
/// let future_a = async { println!("Hello") };
/// let future_b = async { println!("World") };
///
/// let future_list = vec![Task::new(future_a), Task::new(future_b)];
/// ```
///
/// ```rust
/// use pasts::Task;
/// use core::{pin::Pin, future::Future};
///
/// struct Futures {
///     task_a: Task<'static, ()>,
///     // instead of:
///     task_b: Pin<Box<dyn Future<Output = ()> + Send>>,
/// }
///
/// let futures = Futures {
///     task_a: Task::new(async { println!("Hello, world!") }),
///     // instead of:
///     task_b: Box::pin(async { println!("Hello, world!") }),
/// };
/// ```
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
///
/// ```rust
#[doc = include_str!("../examples/tasks.rs")]
/// ```
///
pub struct Task<'a, O = ()>(
    Option<Pin<Box<dyn Future<Output = O> + Send + 'a>>>,
);

impl<O> fmt::Debug for Task<'_, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Task")
    }
}

impl<O> Future for Task<'_, O> {
    type Output = O;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_mut().poll_next(cx)
    }
}

impl<O> Past<O> for Task<'_, O> {
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<O> {
        match self.0 {
            Some(ref mut future) => match Pin::new(future).poll(cx) {
                Pending => Pending,
                Ready(output) => {
                    self.0 = None;
                    Ready(output)
                }
            },
            None => Pending,
        }
    }
}

impl<'a, O> Task<'a, O> {
    /// Create a new type-erased task from a [`Future`].
    pub fn new(future: impl Future<Output = O> + Send + 'a) -> Self {
        Task(Some(Box::pin(future)))
    }
}
