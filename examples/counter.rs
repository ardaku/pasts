#![forbid(unsafe_code)]

use async_std::task;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use pasts::{exec, wait};

/// An event handled by the event loop.
enum Event {
    One(()),
    Two(()),
}

/// Shared state between tasks on the thread.
struct State(usize);

impl State {
    /// Event loop.  Return false to stop program.
    fn event(&mut self, event: Event) {
        match event {
            Event::One(()) => {
                println!("One {}", self.0);
                self.0 += 1;
                if self.0 > 5 {
                    std::process::exit(0);
                }
            }
            Event::Two(()) => {
                println!("Two {}", self.0);
                self.0 += 1
            }
        }
    }
}

struct Interval(Duration, Pin<Box<dyn Future<Output = ()>>>);

impl Interval {
    fn new(duration: Duration) -> Self {
        Interval(duration, Box::pin(task::sleep(duration)))
    }
}

impl Future for &mut Interval {
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

fn main() {
    let mut state = State(0);
    let mut one = Interval::new(Duration::from_secs_f64(0.999));
    let mut two = Interval::new(Duration::from_secs_f64(2.0));

    exec!(state.event(wait! {
        Event::One((&mut one).await),
        Event::Two((&mut two).await),
    }))
}
