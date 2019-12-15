#![forbid(unsafe_code)]

use core::sync::atomic::{AtomicUsize, Ordering};

use pasts::prelude::*;

// A very inefficient interrupt (don't use in production!).
//
// For no_std targets, make your own `Interrupt` that waits for hardware
// interrupts, rather than continuously checking an atomic value in a loop.
struct AtomicInterrupt(AtomicUsize);

impl pasts::Interrupt for AtomicInterrupt {
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

async fn timer_future(duration: std::time::Duration) {
    // On real no_std, you wouldn't be able to use this, and your future would
    // rely on hardware interrupts.
    pasts::spawn_blocking(move || std::thread::sleep(duration)).await
}

fn main() {
    let ret = AtomicInterrupt::block_on(async {
        println!("Waiting 2 seconds…");
        timer_future(std::time::Duration::new(2, 0)).await;
        println!("Waited 2 seconds.");
        "Complete!"
    });
    println!("Future returned: \"{}\"", ret);
}
