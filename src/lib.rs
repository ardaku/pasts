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
//! Some APIs are only available with the **std** feature enabled.  Other APIs
//! only require the **alloc** feature.  APIs that require features are labeled
//! with **feature-name** in their documentation.  You can use no-std with or
//! without the alloc feature (which corresponds to the alloc crate, just as std
//! corresponds to the std crate).  The default is **std** and **alloc**
//! enabled (enabling **std** also enables **alloc**).
//!
//! # Getting Started
//! Add the following to your *Cargo.toml*:
//!
//! ```toml
//! [dependencies]
//! pasts = "0.4"
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
    let mut task_one = one(&state);
    let mut task_two = two(&state);
    let mut tasks = [task_one.fut(), task_two.fut()];
    tasks.select().await;
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

#[cfg(not(feature = "std"))]
extern crate alloc;

/// Re-exported traits
pub mod prelude {
    pub use crate::DynFut;
    pub use crate::Join;
    pub use crate::{Select, SelectOptional};
}

mod dyn_future;
mod executor;
mod join;
mod select;

pub use dyn_future::DynFut;
pub use dyn_future::DynFuture;
pub use executor::{spawn, JoinHandle};
pub use join::Join;
pub use select::{Select, SelectOptional};
