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

/// Type-erased `?`[`Unpin`] + `!`[`Send`] fused future.
///
/// Usage of this type requires an allocator.
///
/// See docs for [`Task`](crate::Task)
pub struct LocalTask<'a, O = ()>(Option<Pin<Box<dyn Future<Output = O> + 'a>>>);

impl<O> fmt::Debug for LocalTask<'_, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LocalTask")
    }
}

impl<O> Future for LocalTask<'_, O> {
    type Output = O;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_mut().poll_next(cx)
    }
}

impl<O> Past<O> for LocalTask<'_, O> {
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

impl<'a, O> LocalTask<'a, O> {
    /// Create a new type-erased task from a [`Future`].
    pub fn new(future: impl Future<Output = O> + 'a) -> Self {
        LocalTask(Some(Box::pin(future)))
    }
}
