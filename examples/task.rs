use core::task::Poll;
use pasts::{Executor, Loop, Task};

type Exit = ();

struct State {
    tasks: Vec<Task<&'static str>>,
}

impl State {
    fn completion(&mut self, id: usize, val: &str) -> Poll<Exit> {
        self.tasks.remove(id);
        println!("Received message from completed task: {}", val);
        if self.tasks.is_empty() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

async fn run() {
    let mut state = State {
        tasks: vec![Box::pin(async { "Hello" }), Box::pin(async { "World" })],
    };

    Loop::new(&mut state)
        .poll(|s| &mut s.tasks, State::completion)
        .await;
}

fn main() {
    Executor::default().block_on(run());
}
