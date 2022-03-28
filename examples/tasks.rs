use pasts::{prelude::*, Loop, Task};

enum Exit {
    /// Task has completed, remove it
    Remove(usize),
}

struct State {}

impl State {
    fn completion(&mut self, (id, val): (usize, &str)) -> Poll<Exit> {
        println!("Received message from completed task: {}", val);

        Ready(Exit::Remove(id))
    }
}

async fn run() {
    let mut state = State {};
    let mut tasks = vec![
        Task::new(|| async { "Hello" }),
        Task::new(|| async { "World" }),
    ];

    while !tasks.is_empty() {
        match Loop::new(&mut state)
            .on(tasks.as_mut_slice(), State::completion)
            .await
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
