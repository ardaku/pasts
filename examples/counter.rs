use core::time::Duration;

use async_std::task::sleep;
use pasts::{prelude::*, Loop};

// Exit type for State.
type Exit = ();

// Shared state between tasks on the thread.
struct State {
    counter: usize,
}

impl State {
    fn one(&mut self, _: ()) -> Poll<Exit> {
        println!("One {}", self.counter);
        self.counter += 1;
        if self.counter > 6 {
            Ready(())
        } else {
            Pending
        }
    }

    fn two(&mut self, _: ()) -> Poll<Exit> {
        println!("Two {}", self.counter);
        self.counter += 1;
        Pending
    }
}

async fn run() {
    let mut state = State { counter: 0 };

    let one = || sleep(Duration::from_secs_f64(1.0));
    let two = || sleep(Duration::from_secs_f64(2.0));

    Loop::new(&mut state)
        .on(one, State::one)
        .on(two, State::two)
        .await;
}

fn main() {
    pasts::block_on(run())
}
