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
    pub use crate::Interrupt;
    pub use crate::Join;
    pub use crate::Select;
}

mod dyn_future;
mod execute;
mod join;
mod select;

pub use dyn_future::DynFut;
pub use dyn_future::DynFuture;
pub use execute::Interrupt;
pub use join::Join;
pub use select::Select;

#[cfg(feature = "std")]
mod spawner;
#[cfg(feature = "std")]
mod thread_interrupt;

#[cfg(feature = "std")]
pub use spawner::spawn_blocking;
#[cfg(feature = "std")]
pub use thread_interrupt::ThreadInterrupt;

// Temporary
#[macro_export]
macro_rules! tasks {
    ($cx:ident while $exit:expr; [ $($gen:ident),* $(,)? ] $(,)?) => {{
        // Create 2 copies of mutable references to futures.
        $(
            let a = &mut $gen($crate::_pasts_hide::ref_from_ptr(&mut $cx));
            let b = $crate::_pasts_hide::ref_from_ptr(a);
            let $gen = (a, b, $gen);
        )*
        // Create generically-typed futures array using first copy.
        $(
            let temp: core::pin::Pin<&mut dyn core::future::Future<Output = _>> = $crate::_pasts_hide::new_pin($gen.0);
            let mut $gen = (temp, $gen.1, $gen.2);
        )*
        let mut tasks_count = 0;
        let mut tasks = [
            $(
                {
                    let temp = &mut $gen.0;
                    tasks_count += 1;
                    temp
                }
            ),*
        ];
        // Create uniquely-typed futures using second copy.
        $(
            let mut $gen = ($crate::_pasts_hide::new_pin($gen.1), $gen.2);
        )*

        while $exit {
            use $crate::Select;

            let (i, ()): (usize, ()) = tasks.select().await;

            tasks_count = 0;
            $({
                if i == tasks_count {
                    $gen.0.set(($gen.1)(pasts::_pasts_hide::ref_from_ptr(&mut $cx)));
                }
                tasks_count += 1;
            })*
        }
    }};

    ($cx:ident; [ $($gen:ident),* $(,)? ] $(,)?) => {{
        tasks!(true, $($generator),*)
    }};
}

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
