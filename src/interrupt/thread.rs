#![allow(clippy::mutex_atomic)]

use crate::_pasts_hide::stn::sync::{Condvar, Mutex};

/// **std** feature required.  An efficient thread interrupt.
///
/// If you can use std, use this `Interrupt`.
pub struct ThreadInterrupt(Mutex<usize>, Condvar);

impl crate::Interrupt for ThreadInterrupt {
    // Initialize the shared data for the interrupt.
    fn new() -> Self {
        ThreadInterrupt(Mutex::new(0), Condvar::new())
    }

    // Interrupt blocking to wake up.
    fn interrupt(&self) {
        // Add 1 to the number of interrupts.
        let mut num = self.0.lock().unwrap();
        *num += 1;

        // We notify the condvar that the value has changed.
        self.1.notify_one();
    }

    // Blocking wait for interrupt, if `Poll::Ready` then stop blocking.
    fn wait_for(&self) {
        // Lock the mutex.
        let mut guard = self.0.lock().unwrap();

        // Reduce by 1 if non-zero.
        if *guard != 0 {
            *guard -= 1;
            if *guard != 0 {
                // After subtraction, still a task waiting - so don't wait.
                return;
            }
        }

        // Wait until not zero (unlock mutex).
        let _guard = self.1.wait(guard).unwrap();
    }
}
