//! Minimal and simpler alternative to the futures crate.
//!
//! - No required std
//! - No allocations
//! - No procedural macros (for faster compile times)
//! - No dependencies
//! - No cost (True zero-cost abstractions!)
//! - No pain (API super easy to learn & use!)
//! - No unsafe code in pinning macro (allowing you to `forbid(unsafe_code)`)

#![no_std]
#![deny(unsafe_code)]
#![warn(missing_docs)]

#[doc(hidden)]
pub mod _pasts_hide {
    #[cfg(feature = "std")]
    pub extern crate std;

    #[cfg(feature = "std")]
    pub use std as stn;

    #[cfg(not(feature = "std"))]
    pub use core as stn;

    /// Not actually safe pinning only for use in `let_pin!()`.
    #[allow(unsafe_code)]
    #[inline(always)]
    pub fn new_pin<P>(pointer: P) -> self::stn::pin::Pin<P>
    where
        P: self::stn::ops::Deref,
    {
        unsafe { self::stn::pin::Pin::new_unchecked(pointer) }
    }
}

mod execute;
mod interrupt;
mod join;
mod pin;
mod select;
mod tasks;
mod run;

/// Re-export of the most common things.
pub mod prelude;

pub use execute::Interrupt;
pub use interrupt::AtomicInterrupt;
pub use tasks::Task;

#[cfg(feature = "std")]
pub use interrupt::CondvarInterrupt;
