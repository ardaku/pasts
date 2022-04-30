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

use crate::prelude::*;

/// Type-erased `?`[`Unpin`] + [`Send`] future.
///
/// Usage of this type requires an allocator.
///
/// # Usage
/// `Task`s are useful for when you:
///  - Need to create a list of [`Future`]s.
///  - Want an abstraction over `Pin<Box<dyn Future<Output = O> + Send>>`
///
/// ```rust
/// let future_a = async { println!("Hello") };
/// let future_b = async { println!("World") };
///
/// let future_list = [Task::new(future_a), Task::new(future_b)].to_vec();
/// ```
///
/// ```rust
/// struct Futures {
///     task_b: Task<'static, ()>,
///     // instead of:
///     task_a: Pin<Box<dyn Future<Output = ()> + Send>>,
/// }
///
/// let futures = Futures {
///     task_a: Task::new(async { println!("Hello, world!") }),
///     // instead of:
///     task_b: Box::pin(async { println!("Hello, world!") }),
/// };
/// ```
///
/// ## Practical Examples
/// Usage with [`Loop`](crate::Loop):
///
/// FIXME: Joining / selecting on futures:
///
/// FIXME: Task spawning:
pub struct Task<'a, O = ()>(Pin<Box<dyn Future<Output = O> + Send + 'a>>);

impl<O> fmt::Debug for Task<'_, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Task")
    }
}

impl<O> Future for Task<'_, O> {
    type Output = O;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().0).poll(cx)
    }
}

impl<'a, O> Task<'a, O> {
    /// Create a new type-erased task from a [`Future`].
    pub fn new(future: impl Future<Output = O> + Send + 'a) -> Self {
        Task(Box::pin(future))
    }
}
