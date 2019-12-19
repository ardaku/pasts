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

    #[inline(always)]
    pub fn new_task<F, O>(
        future: &mut F,
    ) -> (crate::Task<F, O>, stn::mem::MaybeUninit<O>)
    where
        F: self::stn::future::Future<Output = O>,
    {
        (
            crate::Task::new(crate::tasks::new_pin(future)),
            stn::mem::MaybeUninit::uninit(),
        )
    }

    #[allow(unsafe_code)]
    #[inline(always)]
    pub fn join<O>(output: stn::mem::MaybeUninit<O>) -> O {
        unsafe { output.assume_init() }
    }
}

mod execute;
mod join;
mod run;
mod select;
mod tasks;

/// Re-export of traits.
pub mod prelude;

pub use execute::Interrupt;
pub use tasks::Task;

#[cfg(feature = "std")]
mod spawner;
#[cfg(feature = "std")]
mod thread_interrupt;

#[cfg(feature = "std")]
pub use spawner::spawn_blocking;
#[cfg(feature = "std")]
pub use thread_interrupt::ThreadInterrupt;
