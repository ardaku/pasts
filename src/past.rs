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
    iter::{self, RepeatWith},
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

/// Trait for infinite async iteration.  Usually you won't need to use this
/// directly.
///
/// If the underlying type can become disconnected, that should be handled in
/// the future's output (wrapping in [`Option`]).
#[allow(single_use_lifetimes)]
pub trait AsPast<'a, O, F, R>
where
    F: Future<Output = O> + Send + Unpin,
    R: FnMut() -> F,
{
    /// Convert into a [`Past`].
    fn as_past(&'a mut self) -> Past<O, F, R>;
}

impl<'a, T: 'a, O, F, R> AsPast<'a, O, F, R> for T
where
    &'a mut T: IntoIterator<Item = F, IntoIter = RepeatWith<R>>,
    F: Future<Output = O> + Send + Unpin,
    R: FnMut() -> F,
{
    #[inline(always)]
    fn as_past(&'a mut self) -> Past<O, F, R> {
        Past {
            repeater: self.into_iter(),
            future: (),
            _phantom: PhantomData,
        }
    }
}

/// Infinite asynchronous iterator.
#[derive(Debug)]
pub struct Past<O = (), F = crate::Task<O>, R = fn() -> F, M = ()> {
    future: M,
    repeater: RepeatWith<R>,
    // Seriously, not that complicated.
    #[allow(clippy::type_complexity)]
    _phantom: PhantomData<(Pin<Box<F>>, Pin<Box<O>>)>,
}

impl<O, F, R> Past<O, F, R>
where
    R: FnMut() -> F,
    F: Future<Output = O> + Send + Unpin,
{
    /// Create a [`Past`] from a repeating async function/closure.
    #[inline(always)]
    pub fn new(async_fn: R) -> Self {
        Self {
            repeater: iter::repeat_with(async_fn),
            future: (),
            _phantom: PhantomData,
        }
    }

    /// Get a new [`Unpin`] + [`Send`] future ready on next I/O completion.
    // Because this is an "async iterator", and doesn't ever return `None`.
    #[allow(clippy::should_implement_trait)]
    #[inline(always)]
    pub fn next(&mut self) -> F {
        self.repeater.next().unwrap_or_else(|| unreachable!())
    }
}

impl<O, F, R> Past<Pin<Box<F>>, O, R, Pin<Box<F>>>
where
    R: (FnMut() -> F) + Send,
    F: Future<Output = O> + Send,
    O: Send,
{
    /// Create a [`Past`] from a repeating async function/closure.
    ///
    /// Unlike [`Past::new`], the returned future is not required to be
    /// [`Unpin`].  This comes at the cost of a single allocation when this
    /// function is called.
    #[inline(always)]
    pub fn pin(async_fn: R) -> Self {
        let mut repeater = iter::repeat_with(async_fn);

        Past {
            future: Box::pin(repeater.next().unwrap_or_else(|| unreachable!())),
            repeater,
            _phantom: PhantomData,
        }
    }

    /// Get a new [`Unpin`] + [`Send`] future ready on next I/O completion.
    // Because this is an "async iterator", and doesn't ever return `None`.
    #[allow(clippy::should_implement_trait)]
    #[inline(always)]
    pub fn next(&mut self) -> impl Future<Output = O> + Send + Unpin + '_ {
        SendUnpinFuture { past: self }
    }
}

struct SendUnpinFuture<'a, O, F, R>
where
    F: Future<Output = O> + Send,
    R: (FnMut() -> F) + Send,
{
    past: &'a mut Past<Pin<Box<F>>, O, R, Pin<Box<F>>>,
}

impl<O, F, R> Future for SendUnpinFuture<'_, O, F, R>
where
    F: Future<Output = O> + Send,
    R: (FnMut() -> F) + Send,
{
    type Output = O;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<O> {
        let past = &mut self.past;
        past.future.as_mut().poll(cx).map(move |output| {
            let new = past.repeater.next().unwrap_or_else(|| unreachable!());
            past.future.set(new);
            output
        })
    }
}

impl<'a, O, F, R, T> From<&'a mut T> for Past<O, F, R>
where
    T: AsPast<'a, O, F, R>,
    F: Future<Output = O> + Send + Unpin,
    R: FnMut() -> F,
{
    fn from(from: &'a mut T) -> Self {
        from.as_past()
    }
}
