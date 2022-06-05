use pasts::{prelude::*, Join, Fuse};

struct Exit;

struct State {
    tasks: Vec<Fuse<Task<'static, &'static str>>>,
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
            Fuse::from(Box::pin(async { "Hello" }) as Task<'static, _>),
            Fuse::from(Box::pin(async { "World" }) as Task<'static, _>),
        ],
    };

    Join::new(&mut state).on(|s| &mut s.tasks[..], State::completion).await;
}
