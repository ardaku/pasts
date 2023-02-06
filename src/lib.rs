// Copyright © 2019-2023 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// - MIT License (https://mit-license.org/)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).
//
//! Minimal and simpler alternative to the futures crate.
//!
//! # Optional Features
//! Only the _`std`_ feature is enabled by default
//!
//!  - Disable _`std`_ to use pasts without the standard library.
//!  - Enable _`web`_ to use pasts within the javascript DOM.
//!
//! # Getting Started
//!
//! Add the following to your **`./Cargo.toml`**:
//! ```toml
//! [dependencies]
//! pasts = "0.13"
//!
//! ## This example uses async_main for convenience, but it is *not* required to
//! ## use pasts.
//! async_main = { version = "0.2", features = ["pasts"] }
//!
//! ## This example uses async-std for a sleep future, but async-std is *not*
//! ## required to use pasts.
//! async-std = "1.12"
//!
//! ## Also not required for pasts, but allows for portability with WebAssembly
//! ## in the browser.
//! [features]
//! web = ["async_main/web", "pasts/web"]
//! ```
//!
//! ## Multi-Tasking On Multiple Iterators of Futures
//! This example runs two timers in parallel using the `async-std` crate
//! counting from 0 to 6.  The "one" task will always be run for count 6 and
//! stop the program, although which task will run for count 5 may be either
//! "one" or "two" because they trigger at the same time.
//!
//! ```rust,no_run
#![doc = include_str!("../examples/counter.rs")]
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![doc(
    html_logo_url = "https://ardaku.github.io/mm/logo.svg",
    html_favicon_url = "https://ardaku.github.io/mm/icon.svg",
    html_root_url = "https://docs.rs/pasts"
)]
#![forbid(unsafe_code)]
#![warn(
    anonymous_parameters,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_qualifications,
    variant_size_differences
)]

extern crate alloc;

mod join;
mod noti;
mod spawn;

use self::prelude::*;
pub use self::{
    join::Join,
    noti::{Fuse, Loop, Notify, Poller},
    spawn::{Executor, Park, Pool, Spawn},
};

/// An owned dynamically typed [`Notify`] for use in cases where you can’t
/// statically type your result or need to add some indirection.
///
/// **Doesn't work with `one_alloc`**.
pub type BoxNotify<'a, T = ()> =
    Pin<Box<dyn Notify<Event = T> + Send + 'a>>;

/// [`BoxNotify`] without the [`Send`] requirement.
///
/// **Doesn't work with `one_alloc`**.
pub type LocalBoxNotify<'a, T = ()> = Pin<Box<dyn Notify<Event = T> + 'a>>;

impl<T> core::fmt::Debug for LocalBoxNotify<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("LocalBoxNotify")
    }
}

pub mod prelude {
    //! Items that are almost always needed.

    #[doc(no_inline)]
    pub use alloc::boxed::Box;
    #[doc(no_inline)]
    pub use core::{
        future::Future,
        pin::Pin,
        task::{
            Context as Task,
            Poll::{Pending, Ready},
        },
    };

    #[doc(no_inline)]
    pub use crate::{BoxNotify, Fuse, LocalBoxNotify, Notify, Spawn};

    /// Indicates whether a value is available or if the current task has been
    /// scheduled to receive a wakeup instead.
    pub type Poll<T = ()> = core::task::Poll<T>;
}
