#![deny(unsafe_code)]

use pasts::prelude::*;

use core::sync::atomic::{AtomicBool, Ordering};

// A very inefficient executor (don't use in production!).
//
// For no_std targets, make your own `Interrupt` that waits for hardware
// interrupts, rather than continuously checking an atomic value in a loop.
struct AtomicExec(AtomicBool, AtomicBool);

impl AtomicExec {
    const fn new() -> Self {
        AtomicExec(AtomicBool::new(true), AtomicBool::new(false))
    }
}

// Implementing an executor requires unsafe code.
#[allow(unsafe_code)]
impl Executor for AtomicExec {
    // Interrupt blocking to wake up.
    unsafe fn trigger_event(&self) {
        // Add 1 to the number of interrupts.
        self.0.store(true, Ordering::SeqCst);
    }

    // Blocking wait for interrupt, if `Poll::Ready` then stop blocking.
    unsafe fn wait_for_event(&self) {
        // Set used if not already.
        self.1.compare_and_swap(false, true, Ordering::SeqCst);
        // Wait until not zero.
        while self.0.compare_and_swap(true, false, Ordering::SeqCst) == false {}
    }

    fn is_used(&self) -> bool {
        self.1.load(Ordering::SeqCst)
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
