use core::{future::Future, task::Poll, time::Duration};

use async_std::task::sleep;
use pasts::{Loop, Past};

// Exit type for State.
type Exit = ();

// Shared state between tasks on the thread.
struct State<A: Future<Output = ()>, B: Future<Output = ()>> {
    counter: usize,
    one: Past<(), (), A>,
    two: Past<(), (), B>,
}

impl<A: Future<Output = ()>, B: Future<Output = ()>> State<A, B> {
    fn one(&mut self, _: ()) -> Poll<Exit> {
        println!("One {}", self.counter);
        self.counter += 1;
        if self.counter > 6 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    fn two(&mut self, _: ()) -> Poll<Exit> {
        println!("Two {}", self.counter);
        self.counter += 1;
        Poll::Pending
    }
}

async fn run() {
    let mut state = State {
        counter: 0,
        one: Past::pin(|| sleep(Duration::from_secs_f64(1.0))),
        two: Past::pin(|| sleep(Duration::from_secs_f64(2.0))),
    };

    Loop::new(&mut state)
        .when(|s| &mut s.one, State::one)
        .when(|s| &mut s.two, State::two)
        .await;
}

fn main() {
    pasts::block_on(run())
}
