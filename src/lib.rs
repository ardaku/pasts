// Copyright © 2019-2022 The Pasts Contributors.
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
//! The *`std`* feature is enabled by default, disable it to use on no-std.
//!
//! The *`web`* feature is disabled by default, enable it to use pasts within
//! the javascript DOM.
//!
//! The *`no_alloc`* feature is disabled by default, enable it to use pasts on
//! no-std without a global allocator (pulls in
//! [`faux_alloc`](https://crates.io/crates/faux_alloc)).
//!
//! # Getting Started
//!
//! Add the following to your **`./Cargo.toml`**:
//! ```toml
//! autobins = false
//!
//! [[bin]]
//! name = "app"
//! path = "app/main.rs"
//!
//! [dependencies]
//! pasts = "0.11"
//! ## This example uses async-std for a sleep future, but async-std is *not*
//! ## required to use pasts.
//! async-std = "1.11"
//!
//! ## Use web feature when compiling to wasm32-unknown-unknown
//! [target.'cfg(all(target_arch="wasm32",target_os="unknown"))'.dependencies]
//! pasts = { version = "0.11", features = ["web"] }
//! wasm-bindgen = "0.2"
//! ```
//!
//! Create **`./app/main.rs`**:
//! ```rust,no_run
#![doc = include_str!("../gen-docs/app.rs")]
//! ```
//! 
//! ## Multi-Tasking On Multiple Iterators of Futures
//! This example runs two timers in parallel using the `async-std` crate
//! counting from 0 to 6.  The "one" task will always be run for count 6 and
//! stop the program, although which task will run for count 5 may be either
//! "one" or "two" because they trigger at the same time.
//! ```rust,no_run
//! # extern crate alloc;
//! # #[allow(unused_imports)]
//! # use self::main::*;
//! # mod main {
#![doc = include_str!("../gen-docs/counter.rs")]
//! #     pub(super) mod main {
//! #         pub(in crate) async fn main(executor: pasts::Executor) {
//! #             super::main(&executor).await
//! #         }
//! #     }
//! # }
//! # fn main() {
//! #     let executor = pasts::Executor::default();
//! #     executor.spawn(Box::pin(self::main::main::main(executor.clone())));
//! # }
//! ```
//! 
//! <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
//! <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
//! <script>hljs.highlightAll();</script>
//! <style> code.hljs { background-color: #000B; } </style>
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

#[cfg(not(feature = "no_alloc"))]
extern crate alloc;

#[cfg(not(feature = "no_alloc"))]
use alloc as alloc_impl;

#[cfg(feature = "no_alloc")]
use faux_alloc as alloc_impl;

mod exec;
mod join;
mod noti;

use self::prelude::*;
pub use self::{
    exec::{Executor, Sleep},
    join::Join,
    noti::{Fuse, Loop, Notifier, Poller},
};

/// An owned dynamically typed [`Notifier`] for use in cases where you can’t
/// statically type your result or need to add some indirection.
///
/// **Doesn't work with `one_alloc`**.
///
/// <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
/// <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
/// <script>hljs.highlightAll();</script>
/// <style> code.hljs { background-color: #000B; } </style>
pub type Task<'a, T> = Pin<Box<dyn Notifier<Event = T> + Send + 'a>>;

/// [`Task`] without the [`Send`] requirement.
///
/// **Doesn't work with `one_alloc`**.
///
/// <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
/// <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
/// <script>hljs.highlightAll();</script>
/// <style> code.hljs { background-color: #000B; } </style>
pub type Local<'a, T> = Pin<Box<dyn Notifier<Event = T> + 'a>>;

pub mod prelude {
    //! Items that are almost always needed.

    #[doc(no_inline)]
    pub use core::{
        future::Future,
        pin::Pin,
        task::{
            Context as Exec,
            Poll::{self, Pending, Ready},
        },
    };

    #[doc(no_inline)]
    pub use crate::alloc_impl::boxed::Box;
    #[doc(no_inline)]
    pub use crate::{Executor, Fuse, Local, Notifier, Task};
}
