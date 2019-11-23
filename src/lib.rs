//! Minimal and simpler alternative to the futures crate.
//!
//! - No std
//! - No allocations
//! - No dependencies
//! - No cost (True zero-cost abstractions!)
//! - No pain (API super easy to learn & use!)

#![no_std]
#![warn(missing_docs)]

mod wake;
mod pin;
mod execute;

pub use wake::{Wake};
pub use execute::{block_on};
