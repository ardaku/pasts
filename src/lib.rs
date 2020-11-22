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
//! Add the following to your *Cargo.toml*:
//!
//! ```toml
//! [dependencies]
//! pasts = "0.5"
//! aysnc-std = "1.0"
//! ```
//!
#![cfg_attr(
    feature = "std",
    doc = r#"
```rust,no_run
#![forbid(unsafe_code)]

use pasts::prelude::*;
use async_std::task;

use std::{cell::RefCell, time::Duration};

async fn one(state: &RefCell<usize>) {
    println!("Starting task one");
    while *state.borrow() < 5 {
        task::sleep(Duration::new(1, 0)).await;
        let mut state = state.borrow_mut();
        println!("One {}", *state);
        *state += 1;
    }
    println!("Finish task one");
}

async fn two(state: &RefCell<usize>) {
    println!("Starting task two");
    loop {
        task::sleep(Duration::new(2, 0)).await;
        let mut state = state.borrow_mut();
        println!("Two {}", *state);
        *state += 1;
    }
}

async fn example() {
    let state = RefCell::new(0);
    task!(let task_one = one(&state));
    task!(let task_two = two(&state));
    poll![task_one, task_two].await;
}

fn main() {
    pasts::spawn(example);
}
```
"#
)]
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

/// Re-exported traits and macros.
pub mod prelude {
    pub use crate::{poll, task};
}

mod exec;
mod poll;
mod util;

pub use exec::{spawn, JoinHandle};
