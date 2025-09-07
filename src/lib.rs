//! **Minimal asynchronous runtime for Rust**
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
//! pasts = "1.0.0"
//!
//! ## This example uses async_main for convenience, but it is *not* required to
//! ## use pasts.
//! async_main = { version = "0.4.0", features = ["pasts"] }
//!
//! ## This example uses async-std for a sleep future, but async-std is *not*
//! ## required to use pasts.
//! async-std = "1.13.2"
//!
//! ## Also not required for pasts, but allows for portability with WebAssembly
//! ## in the browser.
//! [features]
//! web = ["async_main/web", "pasts/web"]
//! ```

#![no_std]
#![forbid(unsafe_code, missing_docs)]
#![deny(
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_html_tags,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,
    rustdoc::unescaped_backticks,
    rustdoc::redundant_explicit_links
)]
#![warn(
    anonymous_parameters,
    missing_copy_implementations,
    missing_debug_implementations,
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
#![allow(clippy::needless_doctest_main)]
#![doc(
    html_logo_url = "https://ardaku.github.io/mm/logo.svg",
    html_favicon_url = "https://ardaku.github.io/mm/icon.svg",
    html_root_url = "https://docs.rs/pasts"
)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod executor;
mod future;
mod os;
mod park;
mod pool;

pub use self::{
    executor::Executor,
    future::{BoxFuture, LocalBoxFuture},
    park::Park,
    pool::Pool,
};

/// Indicates whether a value is available or if the current task has been
/// scheduled to receive a wakeup instead.
pub type Poll<T = ()> = core::task::Poll<T>;
