// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//
//! Minimal and simpler alternative to the futures crate.
//!
//! # Optional Features
//! The **std** feature is enabled by default, disable it to use on `no_std`.
//!
//! # Getting Started
//! This example pulls in a timer future from the `async-std` crate, then
//! executes it with the pasts executor.
//!
//! Add the following to your *Cargo.toml*:
//!
//! ```toml
//! [dependencies]
//! pasts = "0.6"
//! aysnc-std = "1.0"
//! ```
//!
//! ```rust,no_run
//! use core::{time::Duration, future::Future, task::{Context, Poll}, pin::Pin};
//! use async_std::task;
//! use pasts::{exec, wait};
//!
//! /// An event handled by the event loop.
//! enum Event {
//!     One(()),
//!     Two(()),
//! }
//!
//! /// Shared state between tasks on the thread.
//! struct State(usize);
//!
//! impl State {
//!     /// Event loop.  Return false to stop program.
//!     fn event(&mut self, event: Event) -> bool {
//!         match event {
//!             Event::One(()) => {
//!                 println!("One {}", self.0);
//!                 self.0 += 1;
//!                 if self.0 > 5 {
//!                     return false;
//!                 }
//!             },
//!             Event::Two(()) => {
//!                 println!("Two {}", self.0);
//!                 self.0 += 1
//!             },
//!         }
//!         true
//!     }
//! }
//!
//! struct Interval(Duration, Pin<Box<dyn Future<Output = ()>>>);
//!
//! impl Interval {
//!     fn new(duration: Duration) -> Self {
//!         Interval(duration, Box::pin(task::sleep(duration)))
//!     }
//! }
//!
//! impl Future for &mut Interval {
//!     type Output = ();
//!
//!     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
//!         match self.1.as_mut().poll(cx) {
//!             Poll::Pending => Poll::Pending,
//!             Poll::Ready(()) => {
//!                 self.1 = Box::pin(task::sleep(self.0));
//!                 Poll::Ready(())
//!             }
//!         }
//!     }
//! }
//!
//! fn main() {
//!     let mut state = State(0);
//!     let mut one = Interval::new(Duration::from_secs_f64(0.999));
//!     let mut two = Interval::new(Duration::from_secs_f64(2.0));
//!
//!     exec! { state.event( wait! [
//!         Event::One((&mut one).await),
//!         Event::Two((&mut two).await),
//!     ] .await ) }
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

mod exec;
mod poll;
mod util;

pub use exec::block_on;
