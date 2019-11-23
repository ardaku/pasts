use pasts::Wake;

mod futures;

use crate::futures::TimerFuture;

fn main() {
    // TODO: Eventually generate executor (this function) with macro.
    use std::{
        sync::atomic::{AtomicBool, Ordering},
        task::{Context, Poll},
        future::Future,
    };

    static mut FUTURE_CONDVARS: [AtomicBool; 1] = [AtomicBool::new(true)];

    pub struct FutureZeroTask();

    impl Wake for FutureZeroTask {
        unsafe fn wake_up() {
            FUTURE_CONDVARS[0].store(true, Ordering::Relaxed);
        }
    }

    pasts::let_pin! {
        future_one = async {
            println!("Waiting 2 seconds…");
            TimerFuture::new(std::time::Duration::new(2, 0)).await;
            println!("Done!");
        };
        task = FutureZeroTask();
    };

    // Check for any futures that are ready

    loop {
        if unsafe { FUTURE_CONDVARS[0].load(Ordering::Relaxed) } {
            // This runs whenever woke.
            let task = unsafe { FutureZeroTask::into_waker(&*task) };
            let context = &mut Context::from_waker(&task);
            if let Poll::Pending = future_one.as_mut().poll(context) {
                // Go back to "sleep".
                unsafe {
                    FUTURE_CONDVARS[0].store(false, Ordering::Relaxed);
                }
            } else {
                break;
            }
        }
    }
}
