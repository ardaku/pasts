use {
    pasts::Wake,
    std::{
        future::Future,
        pin::Pin,
        sync::{Arc, Mutex},
        task::{Context, Poll, Waker},
        thread,
        time::Duration,
    },
};

pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();

        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake()
            }
        });

        TimerFuture { shared_state }
    }
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

fn main() {
    // TODO: Eventually generate executor (this function) with macro.
    use std::sync::atomic::{AtomicBool, Ordering};

    static mut FUTURE_CONDVARS: [AtomicBool; 1] = [AtomicBool::new(true)];

    pub struct FutureZeroTask();

    impl Wake for FutureZeroTask {
        fn wake_up() {
            unsafe {
                FUTURE_CONDVARS[0].store(true, Ordering::Relaxed);
            }
        }
    }

    pasts::let_pin! {
        future_one = async {
            println!("Waiting 2 seconds…");
            TimerFuture::new(Duration::new(2, 0)).await;
            println!("Done!");
        };
        task = FutureZeroTask();
    };

    // Check for any futures that are ready

    loop {
        if unsafe { FUTURE_CONDVARS[0].load(Ordering::Relaxed) } {
            // This runs whenever woke.
            let task = FutureZeroTask::into_waker(&*task);
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
