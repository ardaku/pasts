// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::{future::Future, pin::Pin, task::Context, task::Poll};

/// A wrapper around a `Future` trait object.
pub struct DynFuture<'a, T>(&'a mut dyn Future<Output = T>);

impl<T> core::fmt::Debug for DynFuture<'_, T> {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> Result<(), core::fmt::Error> {
        write!(f, "DynFuture")
    }
}

impl<T> Future for DynFuture<'_, T> {
    type Output = T;

    #[allow(unsafe_code)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // unsafe: This is safe because `DynFut` doesn't let you move it.
        let mut fut = unsafe { Pin::new_unchecked(std::ptr::read(&self.0)) };
        let ret = fut.as_mut().poll(cx);
        std::mem::forget(fut);
        ret
    }
}

/// Trait for converting `Future`s into an abstraction of pinned trait objects.
pub trait DynFut<'a, T> {
    /// Turn a future into a generic type.  This is useful for creating an array
    /// of `Future`s.
    fn fut(&'a mut self) -> DynFuture<'a, T>;
}

impl<'a, T, F> DynFut<'a, T> for F
where
    F: Future<Output = T>,
{
    fn fut(&'a mut self) -> DynFuture<'a, T> {
        DynFuture(self)
    }
}
