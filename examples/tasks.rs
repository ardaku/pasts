use pasts::{prelude::*, BoxTask, Join, Task};

enum Exit {
    /// Task has completed, remove it
    Remove(usize),
}

struct State {}

impl State {
    fn completion(&mut self, (id, val): (usize, &str)) -> Poll<Exit> {
        println!("Received message from completed task: {val}");

        Ready(Exit::Remove(id))
    }
}

async fn run() {
    let mut state = State {};

    let mut tasks: Vec<BoxTask<&str>> = vec![
        Task::new(async { "Hello" }).into(),
        Task::new(async { "World" }).into(),
    ];

    while !tasks.is_empty() {
        match Join::new(&mut state).on(&mut tasks[..], State::completion).await
        {
            Exit::Remove(index) => {
                tasks.swap_remove(index);
            }
        }
    }
}

fn main() {
    pasts::block_on(run());
}
