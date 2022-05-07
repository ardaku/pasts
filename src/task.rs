// Copyright © 2019-2022 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use alloc::boxed::Box;
use core::{
    future::Future,
    iter::{self, Once, RepeatWith},
    pin::Pin,
    task::Context,
};

use crate::{past::Past, prelude::*};

pub trait Setter<T: Future>: Unpin {
    fn set(&mut self, some: Option<T>);
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<T::Output>;
}

impl<T: Unpin + Future> Setter<T> for Option<T> {
    fn set(&mut self, some: Option<T>) {
        *self = some;
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<T::Output> {
        if let Some(ref mut future) = self {
            Pin::new(future).poll(cx)
        } else {
            Pending
        }
    }
}

impl<T: Future> Setter<T> for Option<Pin<Box<T>>> {
    fn set(&mut self, some: Option<T>) {
        if let Some(ref mut inner) = self {
            if let Some(value) = some {
                inner.set(value);
                return;
            }
        }
        *self = None;
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<T::Output> {
        if let Some(ref mut future) = self {
            Pin::new(future).poll(cx)
        } else {
            Pending
        }
    }
}

/// Fused asynchronous event producer.
///
/// # Usage
///
/// ```rust
/// use pasts::Task;
/// use core::future::Future;
///
/// let future_a = async { println!("Hello") };
/// let future_b = async { println!("World") };
///
/// let future_list: Vec<Task<dyn Future<Output = ()> + Send>> = vec![
///     Task::new(future_a),
///     Task::new(future_b),
/// ];
///
/// std::thread::spawn(move || {
///     drop(future_list);
/// });
/// ```
///
/// --------
///
/// ```rust
/// use pasts::Task;
/// use core::{pin::Pin, future::Future};
///
/// struct Futures {
///     task_a: Task<dyn Future<Output = ()> + Send>,
///     task_b: Pin<Box<dyn Future<Output = ()> + Send>>,
/// }
///
/// let futures = Futures {
///     task_a: Task::new(async { println!("Hello, world!") }),
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
#[derive(Debug)]
pub struct Task<
    O: ?Sized,
    F: Future = Pin<Box<O>>,
    I: Iterator<Item = F> = Once<F>,
    S: Setter<F> = Option<F>,
> {
    iterator: I,
    future: S,
    _phantom: core::marker::PhantomData<Box<O>>,
}

impl<O: ?Sized, F: Future, I: Iterator<Item = F>, S: Setter<F>>
    Task<O, F, I, S>
{
    /// For integeration with `AsyncIterator` or `Stream` traits
    pub fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<F::Output> {
        <Self as Past<F::Output>>::poll_next(self, cx)
    }
}

impl<O, F: Future + Unpin> Task<O, F> {
    /// Create a new fused asynchronous task from a `Future`.
    pub fn with(future: F) -> Self {
        Task::from(iter::once(future))
    }
}

impl<F: Future> Task<F> {
    /// Create a new fused asynchronous task from a `Future`, pinning on
    /// the heap.
    ///
    /// **Requires non-ZST-exclusive-allocator**
    pub fn with_boxed(future: F) -> Task<F> {
        Task::with(Box::pin(future))
    }
}

impl<F: Future + Unpin, N: FnMut() -> F> Task<(), F, RepeatWith<N>> {
    /// Create a new repeating asynchronous task from a function that returns a
    /// `Future`.
    pub fn with_fn(func: N) -> Self {
        Self::from(iter::repeat_with(func))
    }
}

impl<F: Future, N: FnMut() -> F>
    Task<(), F, RepeatWith<N>, Option<Pin<Box<F>>>>
{
    /// Create a new repeating asynchronous task from a function that returns a
    /// `Future`, pinning on the heap.
    ///
    /// **Requires non-ZST-exclusive-allocator**
    pub fn with_fn_boxed(func: N) -> Self {
        let mut iterator = iter::repeat_with(func);

        Self {
            future: iterator.next().map(Box::pin),
            iterator,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<'a, O> Task<dyn Future<Output = O> + Send + 'a> {
    /// Create a fused asynchronous task from a `Send` future.
    ///
    /// **Requires non-ZST-exclusive-allocator**
    pub fn new(future: impl Future<Output = O> + Send + 'a) -> Self {
        let fut = Box::pin(async { unreachable!() });
        let fut: Pin<Box<dyn Future<Output = O> + Send>> = fut;
        let mut iterator = iter::once(fut);
        let future: Pin<Box<dyn Future<Output = O> + Send>> = Box::pin(future);

        // Iterator should already be exhausted
        iterator.next();

        Self {
            iterator,
            future: Some(future),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<'a, O> Task<dyn Future<Output = O> + 'a> {
    /// Create a `!Send` fused asynchronous task from a `?Send` future.
    ///
    /// **Requires non-ZST-exclusive-allocator**
    pub fn new_local(future: impl Future<Output = O> + 'a) -> Self {
        let fut = Box::pin(async { unreachable!() });
        let fut: Pin<Box<dyn Future<Output = O>>> = fut;
        let mut iterator = iter::once(fut);
        let future: Pin<Box<dyn Future<Output = O>>> = Box::pin(future);

        // Iterator should already be exhausted
        iterator.next();

        Self {
            iterator,
            future: Some(future),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<O, I: Iterator<Item = F>, F: Future + Unpin, T> From<T> for Task<O, F, I>
where
    T: IntoIterator<Item = F, IntoIter = I>,
{
    fn from(other: T) -> Self {
        let mut iterator = other.into_iter();

        Self {
            future: iterator.next(),
            iterator,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<O: ?Sized, F, I, S> Past<F::Output> for Task<O, F, I, S>
where
    F: Future,
    I: Iterator<Item = F>,
    S: Setter<F>,
{
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<F::Output> {
        let output = self.future.poll(cx);
        if output.is_ready() {
            self.future.set(self.iterator.next());
        }
        output
    }
}

impl<O: ?Sized, F, I, S> Past<F::Output> for &mut Task<O, F, I, S>
where
    F: Future,
    I: Iterator<Item = F>,
    S: Setter<F>,
{
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<F::Output> {
        (*self).poll_next(cx)
    }
}
