use core::task::Poll;
use pasts::{Executor, Loop, Task};

type Exit = ();

struct State {
    tasks: [Task<&'static str>; 2],
}

impl State {
    fn completion(&mut self, (id, val): (usize, &str)) -> Poll<Exit> {
        println!("Received message from {}, completed task: {}", id, val);
        Poll::Ready(())
    }
}

async fn run() {
    let mut state = State {
        tasks: [Box::pin(async { "Hello" }), Box::pin(async { "World" })],
    };

    Loop::new(&mut state)
        .poll(|s| &mut s.tasks, State::completion)
        .await;

    std::process::exit(0)
}

fn main() {
    Executor::new().cycle(run())
}
