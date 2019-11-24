//! Minimal and simpler alternative to the futures crate.
//!
//! - No required std
//! - No allocations
//! - No dependencies
//! - No cost (True zero-cost abstractions!)
//! - No pain (API super easy to learn & use!)
//!
//! # Before using no_std (Read this!)
//! This library uses std by default, even though you can disable it.  The
//! reason is because for most cases you won't need no_std and will benefit from
//! having std.  Take for instance the output from Linux command "time" on the
//! timer example program.
//!
//! Output with `default_features = []`:
//! > real	0m2.049s
//!
//! > user	0m2.029s
//!
//! > sys	0m0.016s
//!
//! Output with std enabled:
//! > real	0m2.028s
//!
//! > user	0m0.019s
//!
//! > sys	0m0.010s
//!
//! The user time is a lot lower, this is because with the Rust standard library
//! we can put the thread to sleep (and there is no concept of threads with
//! no_std).  This saves power, improving battery life and allowing the OS to
//! schedule more processing time for other processes.

#![no_std]
#![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;
#[cfg(feature = "std")]
use std as stn;

#[cfg(not(feature = "std"))]
use core as stn;

mod wake;
mod pin;
mod execute;

pub use wake::{Wake};
pub use execute::{block_on};
