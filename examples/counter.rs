#![forbid(unsafe_code)]

use async_std::task;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use pasts::Loop;

// Platform-specific glue code.
pasts::glue!();

/// Shared state between tasks on the thread.
struct State(usize);

impl State {
    fn one(&mut self, _: ()) -> Poll<()> {
        println!("One {}", self.0);
        self.0 += 1;
        if self.0 > 5 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
    
    fn two(&mut self, _: ()) -> Poll<()> {
        println!("Two {}", self.0);
        self.0 += 1;
        Poll::Pending
    }
}

struct Interval(Duration, Pin<Box<dyn Future<Output = ()>>>);

impl Interval {
    fn new(duration: Duration) -> Self {
        Interval(duration, Box::pin(task::sleep(duration)))
    }
}

impl Future for Interval {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        match self.1.as_mut().poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(()) => {
                self.1 = Box::pin(task::sleep(self.0));
                Poll::Ready(())
            }
        }
    }
}

async fn run() {
    let state = State(0);
    let one = Interval::new(Duration::from_secs_f64(0.999));
    let two = Interval::new(Duration::from_secs_f64(2.0));

    Loop::new()
        .when(one, State::one)
        .when(two, State::two)
        .attach(state)
        .await;
}
