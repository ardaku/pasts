use pasts::{prelude::*, BoxTask, Join, Task};

type Exit = ();

struct State {}

impl State {
    fn completion(&mut self, (id, val): (usize, &str)) -> Poll<Exit> {
        println!("Received message from {id}, completed task: {val}");

        Ready(())
    }
}

async fn run() {
    let mut state = State {};

    let tasks: &mut [BoxTask<'static, &str>] = &mut [
        Task::new(async { "Hello" }).into(),
        Task::new(async { "World" }).into(),
    ];

    // First task will complete first.
    Join::new(&mut state).on(tasks, State::completion).await;
}

fn main() {
    pasts::block_on(run())
}
