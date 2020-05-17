#![deny(unsafe_code)]

use pasts::prelude::*;

use core::sync::atomic::{AtomicUsize, Ordering};

// A very inefficient executor (don't use in production!).
//
// For no_std targets, make your own `Interrupt` that waits for hardware
// interrupts, rather than continuously checking an atomic value in a loop.
struct AtomicExec(AtomicUsize);

impl AtomicExec {
    const fn new() -> Self {
        AtomicExec(AtomicUsize::new(0))
    }
}

// Implementing an executor requires unsafe code.
#[allow(unsafe_code)]
impl Executor for AtomicExec {
    // Interrupt blocking to wake up.
    unsafe fn trigger_event(&self) {
        // Add 1 to the number of interrupts.
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    // Blocking wait for interrupt, if `Poll::Ready` then stop blocking.
    unsafe fn wait_for_event(&self) {
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
    static EXECUTOR: AtomicExec = AtomicExec::new();

    let ret = EXECUTOR.block_on(async {
        println!("Waiting 2 seconds…");
        timer_future(std::time::Duration::new(2, 0)).await;
        println!("Waited 2 seconds.");
        "Complete!"
    });
    println!("Future returned: \"{}\"", ret);
}
