use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread::{self, Thread},
};

use super::{Os, Target};

#[derive(Debug)]
pub(crate) struct ParkCx {
    is_parked: AtomicBool,
    thread: Thread,
}

impl Default for ParkCx {
    fn default() -> Self {
        Self {
            is_parked: AtomicBool::new(true),
            thread: thread::current(),
        }
    }
}

impl Target for Os {
    type ParkCx = ParkCx;

    fn park(self, park_cx: &Self::ParkCx) {
        // Loop until old `is_parked` value is `false`, this function will exit
        // immediately on the first call in case `unpark()` was called after the
        // decision to park.
        while park_cx.is_parked.swap(true, Ordering::Relaxed) {
            // Park the thread until either the OS gives a spurious wake up, or
            // `unpark` is called.
            thread::park();
        }
    }

    fn unpark(self, park_cx: &Self::ParkCx) {
        // Unpark the thread, but only if the thread is set to parked.
        if park_cx.is_parked.swap(false, Ordering::Relaxed) {
            park_cx.thread.unpark();
        }
    }
}
