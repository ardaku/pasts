use pasts::{prelude::*, Join, Fuse};

struct Exit;

struct State<'a> {
    tasks: &'a mut [Fuse<Task<'static, &'static str>>],
}

impl State<'_> {
    fn completion(&mut self, (id, val): (usize, &str)) -> Poll<Exit> {
        println!("Received message from {id}, completed task: {val}");

        Ready(Exit)
    }
}

async fn main(_executor: &Executor) {
    let tasks = &mut [
        Fuse::from(Box::pin(async { "Hello" }) as Task<'static, _>),
        Fuse::from(Box::pin(async { "World" }) as Task<'static, _>),
    ];
    let mut state = State { tasks };

    // First task will complete first.
    Join::new(&mut state).on(|s| s.tasks, State::completion).await;
}
