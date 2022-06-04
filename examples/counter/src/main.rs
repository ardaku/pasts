use core::time::Duration;

use async_std::task::sleep;
use pasts::{prelude::*, Join, Loop};

// Exit type for State.
struct Exit;

// Shared state between tasks on the thread.
struct State<'a> {
    counter: usize,
    one: &'a mut (dyn Notifier<Event = ()> + Unpin),
    two: &'a mut (dyn Notifier<Event = ()> + Unpin),
}

impl State<'_> {
    fn one(&mut self, _: ()) -> Poll<Exit> {
        println!("One {}", self.counter);
        self.counter += 1;

        if self.counter > 6 { Ready(Exit) } else { Pending }
    }

    fn two(&mut self, _: ()) -> Poll<Exit> {
        println!("Two {}", self.counter);
        self.counter += 1;

        Pending
    }
}

async fn main(_executor: &Weak<Executor>) {
    let sleep = |seconds| sleep(Duration::from_secs_f64(seconds));
    let one = &mut Loop::pin(|| sleep(1.0));
    let two = &mut Loop::pin(|| sleep(2.0));
    let counter = 0;
    let mut state = State { counter, one, two };

    Join::new(&mut state)
        .on(|s| s.one, State::one)
        .on(|s| s.two, State::two)
        .await;
}
