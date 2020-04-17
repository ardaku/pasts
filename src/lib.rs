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

#[doc(hidden)]
pub mod _pasts_hide {
    #[cfg(feature = "std")]
    pub extern crate std;

    #[cfg(feature = "std")]
    pub use std as stn;

    #[cfg(not(feature = "std"))]
    pub use core as stn;

    /// Not actually safe pinning only for use in macros.
    #[allow(unsafe_code)]
    #[inline(always)]
    pub fn new_pin<P>(pointer: P) -> stn::pin::Pin<P>
    where
        P: stn::ops::Deref,
    {
        unsafe { stn::pin::Pin::new_unchecked(pointer) }
    }

    /// Not actually safe: This is needed for join to return a tuple.
    #[allow(unsafe_code)]
    #[inline(always)]
    pub fn join<O>(output: stn::mem::MaybeUninit<O>) -> O {
        unsafe { output.assume_init() }
    }

    /// Not actually safe: This is needed to create a single-threaded "Mutex" to
    /// satisfy the borrow checker in `run!()`.
    #[allow(unsafe_code)]
    #[inline(always)]
    pub fn ref_from_ptr<'a, T>(ptr: *mut T) -> &'a mut T {
        // Make clippy not complain
        fn deref<'a, T>(ptr: *mut T) -> &'a mut T {
            unsafe { ptr.as_mut().unwrap() }
        }

        deref(ptr)
    }
}

/// Re-exported traits
pub mod prelude {
    pub use crate::Interrupt;
    pub use crate::Select;
    pub use crate::DynFut;
}

mod execute;
mod join;
mod select;

pub use execute::Interrupt;
pub use select::Select;
pub use select::DynFuture; // FIXME: Move
pub use select::DynFut;

#[cfg(feature = "std")]
mod spawner;
#[cfg(feature = "std")]
mod thread_interrupt;

#[cfg(feature = "std")]
pub use spawner::spawn_blocking;
#[cfg(feature = "std")]
pub use thread_interrupt::ThreadInterrupt;
