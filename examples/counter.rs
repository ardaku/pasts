use core::{cell::Cell, iter, time::Duration};

use async_std::task::sleep;
use pasts::{prelude::*, IterAsyncExt, Loop};

// Exit type for State.
type Exit = ();

// Shared state between tasks on the thread.
struct State {
    counter: usize,
}

impl State {
    fn one(&mut self, _: Option<()>) -> Poll<Exit> {
        println!("One {}", self.counter);
        self.counter += 1;
        if self.counter > 6 {
            Ready(())
        } else {
            Pending
        }
    }

    fn two(&mut self, _: Option<()>) -> Poll<Exit> {
        println!("Two {}", self.counter);
        self.counter += 1;
        Pending
    }
}

async fn run() {
    let sleep = |seconds| sleep(Duration::from_secs_f64(seconds));

    let mut state = State { counter: 0 };

    let one = Cell::new(None);
    let mut one = iter::repeat_with(|| sleep(1.0)).boxed(&one);

    let two = Cell::new(None);
    let mut two = iter::repeat_with(|| sleep(2.0)).boxed(&two);

    Loop::new(&mut state)
        .on(&mut one, State::one)
        .on(&mut two, State::two)
        .await;
}

fn main() {
    pasts::block_on(run())
}
