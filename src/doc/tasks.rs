use pasts::{prelude::*, Join};

struct Exit;

struct State {
    tasks: Vec<Task<'static, &'static str>>,
}

impl State {
    fn completion(&mut self, (id, val): (usize, &str)) -> Poll<Exit> {
        println!("Received message from completed task: {val}");

        self.tasks.swap_remove(id);

        if self.tasks.is_empty() { Ready(Exit) } else { Pending }
    }
}

async fn main(_executor: &Executor) {
    let mut state = State {
        tasks: vec![
            Box::pin(async { "Hello" }.fuse()),
            Box::pin(async { "World" }.fuse()),
        ],
    };

    Join::new(&mut state).on(|s| &mut s.tasks[..], State::completion).await;
}
