// Copyright © 2019-2022 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use core::ops::DerefMut;

use crate::{prelude::*, BoxTask, LocalTask};

/// Trait for asynchronous event notification.
///
/// Similar to [`AsyncIterator`](core::async_iter::AsyncIterator), but infinite.
///
/// <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
/// <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
/// <script>hljs.highlightAll();</script>
/// <style> code.hljs { background-color: #000B; } </style>
pub trait Notifier {
    /// The event produced by this notifier
    type Event;

    /// Attempt to get the next value from this iterator, registering a wakeup
    /// when not ready.
    ///
    /// # Return Value
    ///  - `Poll::Pending` - Not ready yet
    ///  - `Poll::Ready(val)` - Ready with next value
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut TaskCx<'_>,
    ) -> Poll<Self::Event>;

    /// Get the next [`Self::Event`]
    ///
    /// # Usage
    /// ```rust
    /// use pasts::{Notifier, prelude::*};
    ///
    /// struct MyAsyncIter;
    ///
    /// impl Notifier for MyAsyncIter {
    ///     type Event = Option<u32>;
    ///
    ///     fn poll_next(
    ///         self: Pin<&mut Self>,
    ///         _cx: &mut TaskCx<'_>
    ///     ) -> Poll<Self::Event> {
    ///         Ready(Some(1))
    ///     }
    /// }
    ///
    /// async fn run() {
    ///     let mut count = 0;
    ///     let mut async_iter = MyAsyncIter;
    ///     let mut iterations = 0;
    ///     while let Some(i) = async_iter.next().await {
    ///         count += i;
    ///         iterations += 1;
    ///         if iterations == 3 {
    ///             break;
    ///         }
    ///     }
    ///     assert_eq!(count, 3);
    /// }
    ///
    /// pasts::Executor::default().spawn(Box::pin(run()));
    /// ```
    fn next(&mut self) -> EventFuture<'_, Self>
    where
        Self: Sized + Unpin,
    {
        EventFuture(self)
    }

    /// Transform produced [`Self::Event`]s with a function.
    fn map<B, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized + Unpin,
    {
        let noti = self;

        Map { noti, f }
    }
}

impl<N: Notifier + Unpin + ?Sized> Notifier for &mut N {
    type Event = N::Event;

    #[inline]
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut TaskCx<'_>,
    ) -> Poll<Self::Event> {
        Pin::new(&mut **self).poll_next(cx)
    }
}

impl<N: Notifier + Unpin> Notifier for [N] {
    type Event = (usize, N::Event);

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut TaskCx<'_>,
    ) -> Poll<Self::Event> {
        for (i, mut this) in self.iter_mut().enumerate() {
            if let Ready(value) = Pin::new(&mut this).poll_next(cx) {
                return Ready((i, value));
            }
        }

        Pending
    }
}

impl<P, N: Notifier + Unpin + ?Sized> Notifier for Pin<P>
where
    P: Unpin + DerefMut<Target = N>,
{
    type Event = N::Event;

    fn poll_next(self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<N::Event> {
        Pin::new(&mut **self.get_mut()).poll_next(cx)
    }
}

#[derive(Debug)]
pub struct EventFuture<'a, N: Notifier + Unpin>(&'a mut N);

impl<N: Notifier + Unpin> Future for EventFuture<'_, N> {
    type Output = N::Event;

    fn poll(self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().0).poll_next(cx)
    }
}

/// A [`Notifier`] created from a function returning [`Poll`].
///
/// <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
/// <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
/// <script>hljs.highlightAll();</script>
/// <style> code.hljs { background-color: #000B; } </style>
#[derive(Debug)]
pub struct PollNextFn<T, F: FnMut(&mut TaskCx<'_>) -> Poll<T> + Unpin>(F);

impl<T, F: FnMut(&mut TaskCx<'_>) -> Poll<T> + Unpin> PollNextFn<T, F> {
    /// Create a new [`Notifier`] from a function returning [`Poll`].
    pub fn new(f: F) -> Self {
        Self(f)
    }
}

impl<T, F> Notifier for PollNextFn<T, F>
where
    F: FnMut(&mut TaskCx<'_>) -> Poll<T> + Unpin,
{
    type Event = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<T> {
        self.0(cx)
    }
}

/// A [`Notifier`] created from a [`Future`].
///
/// The asynchronous equivalent of a thread.
///
/// Requires non-ZST allocator.
///
/// <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
/// <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
/// <script>hljs.highlightAll();</script>
/// <style> code.hljs { background-color: #000B; } </style>
#[derive(Debug)]
pub struct Task<F: Future + ?Sized>(Option<Pin<Box<F>>>);

impl<F: Future> Task<F> {
    /// Create a fused [`Notifier`] from a [`Future`]
    pub fn new(future: F) -> Self {
        Self(Some(Box::pin(future)))
    }
}

impl<F: Future + Send + 'static> From<Task<F>> for BoxTask<'_, F::Output> {
    fn from(other: Task<F>) -> Self {
        Task(other.0.map(|x| -> Pin<Box<dyn Future<Output = _> + Send>> { x }))
    }
}

impl<F: Future + 'static> From<Task<F>> for LocalTask<'_, F::Output> {
    fn from(other: Task<F>) -> Self {
        Task(other.0.map(|x| -> Pin<Box<dyn Future<Output = _>>> { x }))
    }
}

impl<F: Future + ?Sized> Notifier for Task<F> {
    type Event = F::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<F::Output> {
        let this = self.get_mut();
        if let Some(ref mut future) = this.0 {
            return Pin::new(future).poll(cx).map(|event| {
                this.0 = None;
                event
            });
        }

        Pending
    }
}

pub trait Looper<F: Future>: Unpin {
    fn poll(&mut self, cx: &mut TaskCx<'_>) -> Poll<F::Output>;
    fn set(&mut self, future: F);
}

impl<F: Future> Looper<F> for Pin<Box<F>> {
    fn poll(&mut self, cx: &mut TaskCx<'_>) -> Poll<F::Output> {
        Pin::new(self).poll(cx)
    }

    fn set(&mut self, f: F) {
        self.set(f);
    }
}

impl<F: Future + Unpin> Looper<F> for F {
    fn poll(&mut self, cx: &mut TaskCx<'_>) -> Poll<F::Output> {
        Pin::new(self).poll(cx)
    }

    fn set(&mut self, f: F) {
        *self = f;
    }
}

/// A [`Notifier`] created from a function returning [`Future`]s.
///
/// A repeating [`Task`].
///
/// <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
/// <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
/// <script>hljs.highlightAll();</script>
/// <style> code.hljs { background-color: #000B; } </style>
#[derive(Debug)]
pub struct Loop<F: Future, L: FnMut() -> F + Unpin, S>(S, L);

impl<F: Future, L: FnMut() -> F + Unpin> Loop<F, L, F> {
    /// Create a fused [`Notifier`] from an [`Unpin`] [`Future`]
    pub fn new(mut looper: L) -> Self
    where
        F: Unpin,
    {
        Self(looper(), looper)
    }
}

impl<F: Future, L: FnMut() -> F + Unpin> Loop<F, L, Pin<Box<F>>> {
    /// Create a fused [`Notifier`] from a `!Unpin` [`Future`]
    ///
    /// Requires non-ZST allocator.
    pub fn pin(mut looper: L) -> Self {
        Self(Box::pin(looper()), looper)
    }
}

impl<F: Future, L, S: Looper<F>> Notifier for Loop<F, L, S>
where
    L: FnMut() -> F + Unpin,
{
    type Event = F::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<F::Output> {
        let this = self.get_mut();
        let poll = Pin::new(&mut this.0).poll(cx);

        if poll.is_ready() {
            this.0.set(this.1());
        }

        poll
    }
}

/// A notifier returned from [`Notifier::map()`].
#[derive(Debug)]
pub struct Map<I, F> {
    noti: I,
    f: F,
}

impl<E, N: Notifier + Unpin, F> Notifier for Map<N, F>
where
    F: FnMut(N::Event) -> E + Unpin,
{
    type Event = E;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<E> {
        Pin::new(&mut self.noti).poll_next(cx).map(&mut self.f)
    }
}
