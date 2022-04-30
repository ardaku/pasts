// Copyright © 2019-2022 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).
//
//! Minimal and simpler alternative to the futures crate.
//!
//! # Optional Features
//! The **std** feature is enabled by default, disable it to use on `no_std`.
//!
//! # Getting Started
//! This example runs two timers in parallel using the `async-std` crate
//! counting from 0 to 6.  The "one" task will always be run for count 6 and
//! stop the program, although which task will run for count 5 may be either
//! "one" or "two" because they trigger at the same time.
//!
//! Add the following to your *Cargo.toml*:
//!
//! ```toml
//! [dependencies]
//! pasts = "0.8"
//! aysnc-std = "1.0"
//! ```
//!
//! ```rust,no_run
//! use core::time::Duration;
//!
//! use async_std::task::sleep;
//! use pasts::{prelude::*, Loop};
//!
//! // Exit type for State.
//! type Exit = ();
//!
//! // Shared state between tasks on the thread.
//! struct State {
//!     counter: usize,
//! }
//!
//! impl State {
//!     fn one(&mut self, _: ()) -> Poll<Exit> {
//!         println!("One {}", self.counter);
//!         self.counter += 1;
//!         if self.counter > 6 {
//!             Ready(())
//!         } else {
//!             Pending
//!         }
//!     }
//!
//!     fn two(&mut self, _: ()) -> Poll<Exit> {
//!         println!("Two {}", self.counter);
//!         self.counter += 1;
//!         Pending
//!     }
//! }
//!
//! async fn run() {
//!     let mut state = State { counter: 0 };
//!
//!     let one = || sleep(Duration::from_secs_f64(1.0));
//!     let two = || sleep(Duration::from_secs_f64(2.0));
//!
//!     Loop::new(&mut state)
//!         .on(one, State::one)
//!         .on(two, State::two)
//!         .await;
//! }
//!
//! fn main() {
//!     pasts::block_on(run())
//! }
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

mod exec;
mod func;
mod iter;
mod past;
mod task;

pub use exec::{block_on, BlockOn, Executor};
pub use func::{poll_fn, poll_next_fn};
pub use iter::IterAsyncExt;
pub use past::Loop;
pub use task::Task;

pub mod prelude {
    //! Items that are almost always needed.
    //!
    //! Includes [`Poll`], [`Poll::Pending`], and [`Poll::Ready`] because they
    //! are essentially the asynchronous equivalent of [`Option`] which is in
    //! the prelude, though more geared towards control flow.

    pub use core::task::Poll::{self, Pending, Ready};
}
