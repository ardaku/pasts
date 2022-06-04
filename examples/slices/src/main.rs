use pasts::{prelude::*, BoxTask, Join, Task};

struct Exit;

struct State<'a> {
    tasks: &'a mut [BoxTask<'static, &'static str>],
}

impl State<'_> {
    fn completion(&mut self, (id, val): (usize, &str)) -> Poll<Exit> {
        println!("Received message from {id}, completed task: {val}");

        Ready(Exit)
    }
}

async fn main(_executor: &Executor) {
    let tasks = &mut [
        Task::new(async { "Hello" }).into(),
        Task::new(async { "World" }).into(),
    ];
    let mut state = State { tasks };

    // First task will complete first.
    Join::new(&mut state).on(|s| s.tasks, State::completion).await;
}
