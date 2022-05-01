// Copyright © 2019-2022 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use alloc::boxed::Box;

use core::{cell::Cell, fmt, future::Future, pin::Pin, task::Context};

use crate::prelude::*;

pub struct BoxAsyncIter<'a, F, I: Iterator<Item = F>> {
    future: &'a Cell<Option<Pin<Box<F>>>>,
    iter: I,
}

impl<F, I: Iterator<Item = F>> fmt::Debug for BoxAsyncIter<'_, F, I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BoxAsyncIter")
    }
}

pub struct BoxAsyncFut<'a, F>(&'a Cell<Option<Pin<Box<F>>>>);

impl<F> fmt::Debug for BoxAsyncFut<'_, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BoxAsyncFut")
    }
}

impl<F: Future> Future for BoxAsyncFut<'_, F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0
            .take()
            .map(|mut fut| {
                let poll = Pin::new(&mut fut).poll(cx);
                self.0.set(Some(fut));
                poll
            })
            .unwrap_or(Pending)
    }
}

// Extra lifetime indirection can be removed once GATs are stabilized
impl<'a, F, I: Iterator<Item = F>> Iterator for &'a mut BoxAsyncIter<'_, F, I> {
    type Item = BoxAsyncFut<'a, F>;

    fn next(&mut self) -> Option<Self::Item> {
        let future = self.future.take();
        if future
            .and_then(|mut fut| {
                self.iter.next().map(|next| {
                    fut.set(next);
                    self.future.set(Some(fut));
                })
            })
            .is_some()
        {
            Some(BoxAsyncFut(self.future))
        } else {
            None
        }
    }
}

/// Async iteration extensions.
///
/// This trait adds a `boxed()` method to iterators, which pins and transforms
/// `!`[`Unpin`] futures into [`Unpin`] futures with a one-time allocation.
///
/// Can't be used on a ZST allocator.
pub trait IterAsyncExt: Iterator + Sized {
    /// Re-use a heap allocation for `!`[`Unpin`] futures to make them [`Unpin`]
    fn boxed(
        mut self,
        future_state: &Cell<Option<Pin<Box<Self::Item>>>>,
    ) -> BoxAsyncIter<'_, Self::Item, Self> {
        future_state.set(self.next().map(Box::pin));

        BoxAsyncIter {
            future: future_state,
            iter: self,
        }
    }
}

impl<T> IterAsyncExt for T where T: Iterator + Sized {}
