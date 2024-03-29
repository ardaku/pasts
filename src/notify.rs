//! Asynchronous event notifys
//!
//! A [`Notify`] is kind of like a cross between a [`Future`] and an
//! [`AsyncIterator`](core::async_iter::AsyncIterator).  Like streams, they may
//! return more than one value, and are expected to not panic after polling.
//! Like futures, they produce non-optional values.  In a sense they are an
//! infinite stream.  In another sense, they are a repeating future.
//!
//! # Why Another Abstraction?
//! Notifys allow for some nice ergonomics and guarantees when working with
//! event-loop based asynchronous code, which could lead to some
//! simplifications.  Unlike futures and streams, they do not need to be fused,
//! and if your stream is infinite, you won't need to sprinkle `unwrap()`s in
//! your code at each call to `.next()`.  They also lend themselves nicely for
//! creating clean and simple multimedia APIs.

use core::fmt;

use crate::prelude::*;

/// An owned dynamically typed [`Notify`] for use in cases where you can’t
/// statically type your result or need to add some indirection.
pub type BoxNotify<'a, T = ()> = Pin<Box<dyn Notify<Event = T> + Send + 'a>>;

impl<T> fmt::Debug for BoxNotify<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BoxNotify")
    }
}

/// [`BoxNotify`] without the [`Send`] requirement.
pub type LocalBoxNotify<'a, T = ()> = Pin<Box<dyn Notify<Event = T> + 'a>>;

impl<T> fmt::Debug for LocalBoxNotify<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LocalBoxNotify")
    }
}

/// Trait for asynchronous event notification
///
/// Similar to [`AsyncIterator`](core::async_iter::AsyncIterator), but infinite.
///
/// It's expected that [`Notify`]s can be polled indefinitely without causing
/// panics or undefined behavior.  They are not required to continue sending
/// events indefinitely, though.
pub trait Notify {
    /// The event produced by this notify
    type Event;

    /// Get the next event from this notify, registering a wakeup when not
    /// ready.
    ///
    /// # Return Value
    ///  - `Poll::Pending` - Not ready yet
    ///  - `Poll::Ready(val)` - Ready with next value
    fn poll_next(self: Pin<&mut Self>, t: &mut Task<'_>) -> Poll<Self::Event>;
}

impl<N> Notify for Box<N>
where
    N: ?Sized + Notify + Unpin,
{
    type Event = N::Event;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, t: &mut Task<'_>) -> Poll<N::Event> {
        Pin::new(self.get_mut().as_mut()).poll_next(t)
    }
}

impl<N, P> Notify for Pin<P>
where
    P: core::ops::DerefMut<Target = N> + Unpin,
    N: Notify + ?Sized,
{
    type Event = N::Event;

    fn poll_next(self: Pin<&mut Self>, t: &mut Task<'_>) -> Poll<Self::Event> {
        Pin::get_mut(self).as_mut().poll_next(t)
    }
}

impl<N> Notify for &mut N
where
    N: Notify + Unpin + ?Sized,
{
    type Event = N::Event;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, t: &mut Task<'_>) -> Poll<N::Event> {
        Pin::new(&mut **self).poll_next(t)
    }
}

impl<N> Notify for [N]
where
    N: Notify + Unpin,
{
    type Event = (usize, N::Event);

    #[inline]
    fn poll_next(self: Pin<&mut Self>, t: &mut Task<'_>) -> Poll<Self::Event> {
        for (i, this) in self.get_mut().iter_mut().enumerate() {
            if let Poll::Ready(value) = Pin::new(this).poll_next(t) {
                return Poll::Ready((i, value));
            }
        }

        Poll::Pending
    }
}

/// An extension trait for [`Notify`]s that provides a variety of convenient
/// adapters.
pub trait NotifyExt: Notify + Sized + Unpin {
    /// Get the next [`Notify::Event`]
    ///
    /// # Usage
    /// ```rust
    /// use pasts::prelude::*;
    /// use async_main::Spawn;
    ///
    /// struct MyAsyncIter;
    ///
    /// impl Notify for MyAsyncIter {
    ///     type Event = Option<u32>;
    ///
    ///     fn poll_next(self: Pin<&mut Self>, _: &mut Task<'_>) -> Poll<Self::Event> {
    ///         Ready(Some(1))
    ///     }
    /// }
    ///
    /// #[async_main::async_main]
    /// async fn main(_spawner: impl Spawn) {
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
    /// ```
    #[inline(always)]
    fn next(&mut self) -> Next<'_, Self> {
        Next(self)
    }

    /// Transform produced [`Notify::Event`]s with a function.
    #[inline(always)]
    fn map<F>(self, f: F) -> Map<Self, F> {
        let noti = self;

        Map { noti, f }
    }
}

impl<N: Notify + Sized + Unpin> NotifyExt for N {}

/// The [`Future`] returned from [`NotifyExt::next()`]
#[derive(Debug)]
pub struct Next<'a, N>(&'a mut N)
where
    N: Notify + Unpin;

impl<N> Future for Next<'_, N>
where
    N: Notify + Unpin,
{
    type Output = N::Event;

    #[inline]
    fn poll(self: Pin<&mut Self>, t: &mut Task<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().0).poll_next(t)
    }
}

/// Trait for "fusing" a [`Future`] (conversion to a [`Notify`])
pub trait Fuse: Sized {
    /// Fuse the [`Future`]
    fn fuse(self) -> Option<Self>;
}

impl<F> Fuse for F
where
    F: Future,
{
    fn fuse(self) -> Option<Self> {
        self.into()
    }
}

impl<F: Future> Notify for Option<F> {
    type Event = F::Output;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, t: &mut Task<'_>) -> Poll<F::Output> {
        let mut s = self;
        let out = s.as_mut().as_pin_mut().map(|f| f.poll(t));
        if matches!(out, Some(Poll::Ready(_))) {
            s.set(None);
        }
        out.unwrap_or(Poll::Pending)
    }
}

/// The [`Notify`] returned from [`NotifyExt::map()`]
#[derive(Debug)]
pub struct Map<N, F> {
    noti: N,
    f: F,
}

impl<N, F, E> Notify for Map<N, F>
where
    N: Notify + Unpin,
    F: FnMut(N::Event) -> E + Unpin,
{
    type Event = E;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, t: &mut Task<'_>) -> Poll<E> {
        Pin::new(&mut self.noti).poll_next(t).map(&mut self.f)
    }
}

/// A [`Notify`] that wraps a function returning a [`Future`]
///
/// This struct is created by [`future_fn()`].  See its documentation for more.
#[derive(Debug)]
pub struct FutureFn<T, F>(T, F);

impl<T, F> Notify for FutureFn<T, F>
where
    T: Future + Unpin,
    F: FnMut() -> T + Unpin,
{
    type Event = T::Output;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, t: &mut Task<'_>) -> Poll<T::Output> {
        let this = self.get_mut();
        let poll = Pin::new(&mut this.0).poll(t);

        if poll.is_ready() {
            Pin::new(&mut this.0).set(this.1());
        }

        poll
    }
}

/// A [`Notify`] created from a function returning [`Poll`]
#[derive(Debug)]
pub struct PollFn<F>(F);

impl<T, F> Notify for PollFn<F>
where
    F: FnMut(&mut Task<'_>) -> Poll<T> + Unpin,
{
    type Event = T;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, t: &mut Task<'_>) -> Poll<T> {
        self.get_mut().0(t)
    }
}

/// A [`Notify`] that never produces an event.
///
/// This struct is created by [`pending()`].  See its documentation for more.
#[derive(Debug)]
pub struct Pending<T>(core::marker::PhantomData<fn() -> T>);

impl<T> Notify for Pending<T> {
    type Event = T;

    fn poll_next(self: Pin<&mut Self>, _task: &mut Task<'_>) -> Poll<T> {
        Poll::Pending
    }
}

/// A [`Notify`] that immediately produces a single event.
///
/// This struct is created by [`ready()`].  See its documentation for more.
#[derive(Debug)]
pub struct Ready<T: Unpin>(Option<T>);

impl<T: Unpin> Notify for Ready<T> {
    type Event = T;

    fn poll_next(self: Pin<&mut Self>, _task: &mut Task<'_>) -> Poll<T> {
        let Some(event) = self.get_mut().0.take() else {
            return Poll::Pending;
        };

        Poll::Ready(event)
    }
}

/// A [`Notify`] that selects over a list of [`Notify`]s
///
/// This struct is created by [`select()`].  See its documentation for more.
pub struct Select<'a, E, const N: usize>(
    [&'a mut (dyn Notify<Event = E> + Unpin); N],
    usize,
);

impl<E, const N: usize> fmt::Debug for Select<'_, E, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Select")
    }
}

impl<E: Unpin, const N: usize> Notify for Select<'_, E, N> {
    type Event = E;

    fn poll_next(self: Pin<&mut Self>, task: &mut Task<'_>) -> Poll<E> {
        let s = self.get_mut();
        let start = s.1;

        // Early exit for if there is nothing that can be ready.
        if N == 0 {
            return Poll::Pending;
        }

        for i in (start..N).chain(0..start) {
            if let Poll::Ready(event) = Pin::new(&mut s.0[i]).poll_next(task) {
                return Poll::Ready(event);
            }
        }

        Poll::Pending
    }
}

/// Create a [`Notify`] that wraps a function returning a [`Future`].
///
/// Polling the notify delegates to future returned by the wrapped function.
/// The wrapped function is called immediately, and is only called again once
/// the future is polled and returns `Ready`.
pub fn future_fn<T, F>(mut f: F) -> FutureFn<T, F>
where
    T: Future + Unpin,
    F: FnMut() -> T + Unpin,
{
    FutureFn(f(), f)
}

/// Create a [`Notify`] that wraps a function returning [`Poll`].
///
/// Polling the future delegates to the wrapped function.
pub fn poll_fn<T, F>(f: F) -> PollFn<F>
where
    F: FnMut(&mut Task<'_>) -> Poll<T> + Unpin,
{
    PollFn(f)
}

/// Create a [`Notify`] which never becomes ready with an event.
pub fn pending<T>() -> Pending<T> {
    Pending(core::marker::PhantomData)
}

/// Create a [`Notify`] which is immediately ready with an event.
pub fn ready<T: Unpin>(t: T) -> Ready<T> {
    Ready(t.into())
}

/// Create a [`Notify`] that selects over a list of [`Notify`]s.
pub fn select<E, const N: usize>(
    notifys: [&mut (dyn Notify<Event = E> + Unpin); N],
) -> Select<'_, E, N> {
    Select(notifys, 0)
}
