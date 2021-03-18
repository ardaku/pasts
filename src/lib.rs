// Pasts
// Copyright © 2019-2021 Jeron Aldaron Lau.
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
//! pasts = "0.7"
//! aysnc-std = "1.0"
//! ```
//!
//! ```rust,no_run
//! use async_std::task::sleep;
//! use core::future::Future;
//! use core::pin::Pin;
//! use core::task::{Context, Poll};
//! use core::time::Duration;
//! use pasts::Loop;
//!
//! /// Shared state between tasks on the thread.
//! struct State(usize);
//!
//! impl State {
//!     fn one(&mut self, _: ()) -> Poll<()> {
//!         println!("One {}", self.0);
//!         self.0 += 1;
//!         if self.0 > 6 {
//!             Poll::Ready(())
//!         } else {
//!             Poll::Pending
//!         }
//!     }
//!
//!     fn two(&mut self, _: ()) -> Poll<()> {
//!         println!("Two {}", self.0);
//!         self.0 += 1;
//!         Poll::Pending
//!     }
//! }
//!
//! struct Interval(Duration, Pin<Box<dyn Future<Output = ()>>>);
//!
//! impl Interval {
//!     fn new(duration: Duration) -> Self {
//!         Interval(duration, Box::pin(sleep(duration)))
//!     }
//! }
//!
//! impl Future for Interval {
//!     type Output = ();
//!
//!     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
//!         match self.1.as_mut().poll(cx) {
//!             Poll::Pending => Poll::Pending,
//!             Poll::Ready(()) => {
//!                 self.1 = Box::pin(sleep(self.0));
//!                 Poll::Ready(())
//!             }
//!         }
//!     }
//! }
//!
//! async fn run() {
//!     let state = State(0);
//!     let one = Interval::new(Duration::from_secs_f64(1.0));
//!     let two = Interval::new(Duration::from_secs_f64(2.0));
//!
//!     Loop::new(state)
//!         .when(one, State::one)
//!         .when(two, State::two)
//!         .await;
//! }
//!
//! fn main() {
//!     pasts::block_on(run())
//! }
//! ```
#![cfg_attr(not(feature = "std"), no_std)]
#![doc(
    html_logo_url = "https://libcala.github.io/logo.svg",
    html_favicon_url = "https://libcala.github.io/icon.svg",
    html_root_url = "https://docs.rs/pasts"
)]
#![deny(unsafe_code)]
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

#[cfg(any(not(feature = "std"), target_arch = "wasm32"))]
extern crate alloc;

mod r#exec;
mod r#loop;
mod r#poll;
mod r#task;
mod r#util;

pub use r#exec::block_on;
pub use r#loop::Loop;
pub use r#poll::Poll;
pub use r#task::Task;
