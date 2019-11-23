//! Minimal and simpler alternative to the futures crate.

#![no_std]
#![warn(missing_docs)]

mod wake;
mod pin;
mod execute;

pub use wake::{Wake};
pub use execute::{block_on};
