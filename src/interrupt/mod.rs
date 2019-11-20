use crate::_pasts_hide::stn::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "std")]
mod condvar;
#[cfg(feature = "std")]
pub use self::condvar::CondvarInterrupt;

/// A very inefficient interrupt (only use for testing).  On no_std, make your
/// own `Interrupt` that waits for hardware interrupts, rather than continuously
/// checking an atomic value in a loop.
pub struct AtomicInterrupt(AtomicUsize);

impl crate::Interrupt for AtomicInterrupt {
    // Initialize the shared data for the interrupt.
    fn new() -> Self {
        AtomicInterrupt(AtomicUsize::new(0))
    }

    // Interrupt blocking to wake up.
    fn interrupt(&self) {
        // Add 1 to the number of interrupts.
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    // Blocking wait for interrupt, if `Poll::Ready` then stop blocking.
    fn wait_for(&self) {
        // Reduce by 1 if non-zero.
        self.0.compare_and_swap(0, 1, Ordering::Relaxed);
        self.0.fetch_sub(1, Ordering::Relaxed);

        // Wait until not zero.
        while self.0.load(Ordering::Relaxed) == 0 {}
    }
}
