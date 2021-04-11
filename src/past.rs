// Pasts
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

#[cfg(any(target_arch = "wasm32", not(feature = "std")))]
use alloc::vec::Vec;

/// Trait for abstracting over unpin futures and slices of unpin futures.
pub trait Past<T>: Unpin {
    /// The poll method for polling one or multiple futures.
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<T>;
}

impl<T, F: Future<Output = T> + Unpin> Past<T> for Option<F> {
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<T> {
        if let Some(future) = self {
            if let Poll::Ready(output) = Pin::new(future).poll(cx) {
                return Poll::Ready(output);
            }
        }
        Poll::Pending
    }
}

impl<T, F: Future<Output = T> + Unpin> Past<(usize, T)> for Vec<F> {
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<(usize, T)> {
        for (i, future) in self.iter_mut().enumerate() {
            if let Poll::Ready(output) = Pin::new(future).poll(cx) {
                return Poll::Ready((i, output));
            }
        }
        Poll::Pending
    }
}

impl<T, F: Future<Output = T> + Unpin, const G: usize> Past<(usize, T)>
    for [F; G]
{
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<(usize, T)> {
        for (i, future) in self.iter_mut().enumerate() {
            if let Poll::Ready(output) = Pin::new(future).poll(cx) {
                return Poll::Ready((i, output));
            }
        }
        Poll::Pending
    }
}
