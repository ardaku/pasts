// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![cfg_attr(feature = "docs-rs", feature(external_doc))]
#![cfg_attr(feature = "docs-rs", doc(include = "../README.md"))]
#![doc = ""]
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

/// Re-exported traits
pub mod prelude {
    pub use crate::DynFut;
    pub use crate::Executor;
    pub use crate::Join;
    pub use crate::Select;
}

mod dyn_future;
mod executor;
mod join;
mod select;

pub use dyn_future::DynFut;
pub use dyn_future::DynFuture;
pub use executor::Executor;
pub use join::Join;
pub use select::Select;

#[cfg(feature = "std")]
mod spawner;
#[cfg(feature = "std")]
mod cvar_exec;

#[cfg(feature = "std")]
pub use spawner::spawn_blocking;
#[cfg(feature = "std")]
pub use cvar_exec::CvarExec;
